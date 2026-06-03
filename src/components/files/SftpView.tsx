import { useState, useEffect, useCallback, useRef } from 'react';
import {
  Folder,
  File,
  ArrowUp,
  Upload,
  Download,
  Trash2,
  Edit,
  Plus,
  Loader2,
  HardDrive,
  Globe,
} from 'lucide-react';
import type { SftpEntry } from '../../types';
import { useAppStore } from '../../stores/appStore';
import {
  sftpListDir,
  sftpReadFile,
  sftpWriteFile,
  sftpRemoveFile,
  sftpRename,
  sftpMkdir,
  sftpChmod,
  connectTerminal,
  listLocalDir,
  writeLocalFile,
  removeLocalFile,
  renameLocalFile,
  createLocalDir,
} from '../../utils/tauri';

// ---------- helpers ----------

function formatSize(bytes: number): string {
  if (bytes < 0) return '-';
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const val = bytes / Math.pow(1024, i);
  return `${val.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

function formatPermissions(perms: string): string {
  // Backend may send octal like "755" or rwx string. Normalise to rwxrwxrwx.
  if (perms.length === 10 && /[rwx\-]/.test(perms)) return perms; // already rwx
  const num = parseInt(perms, 8);
  if (isNaN(num)) return perms;
  const chars = ['---', '--x', '-w-', '-wx', 'r--', 'r-x', 'rw-', 'rwx'];
  return chars[(num >> 6) & 7] + chars[(num >> 3) & 7] + chars[num & 7];
}

function formatDate(iso: string): string {
  try {
    const d = new Date(iso);
    if (isNaN(d.getTime())) return iso;
    const pad = (n: number) => String(n).padStart(2, '0');
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
  } catch {
    return iso;
  }
}

/** Split a path into breadcrumb segments. */
function pathSegments(p: string): { label: string; path: string }[] {
  if (!p || p === '/') return [{ label: '/', path: '/' }];
  const parts = p.split('/').filter(Boolean);
  const segs: { label: string; path: string }[] = [{ label: '/', path: '/' }];
  let acc = '';
  for (const part of parts) {
    acc += '/' + part;
    segs.push({ label: part, path: acc });
  }
  return segs;
}

function joinPath(parent: string, name: string): string {
  return parent === '/' ? `/${name}` : `${parent.replace(/\/+$/, '')}/${name}`;
}

/** Sort entries: directories first, then alphabetical by name. */
function sortEntries(entries: SftpEntry[]): SftpEntry[] {
  return [...entries].sort((a, b) => {
    if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
    return a.name.localeCompare(b.name);
  });
}

// ---------- context menu ----------

interface ContextMenuState {
  visible: boolean;
  x: number;
  y: number;
  entry: SftpEntry | null; // null = background right-click
}

const initialCtx: ContextMenuState = { visible: false, x: 0, y: 0, entry: null };
const MAX_TEXT_EDIT_BYTES = 1024 * 1024;

// ---------- component ----------

interface SftpViewProps {
  sessionId?: string;
}

type PanelSide = 'remote' | 'local';

export default function SftpView({ sessionId: sessionIdProp }: SftpViewProps) {
  const activeTab = useAppStore((s) => s.tabs.find((t) => t.id === s.activeTabId));
  const updateTab = useAppStore((s) => s.updateTab);
  const sessionId = sessionIdProp || activeTab?.sessionId || '';
  const connectStartedRef = useRef(false);

  // Remote state
  const [remotePath, setRemotePath] = useState('/');
  const [remoteEntries, setRemoteEntries] = useState<SftpEntry[]>([]);
  const [remoteLoading, setRemoteLoading] = useState(false);
  const [remoteError, setRemoteError] = useState<string | null>(null);
  const [sessionLoading, setSessionLoading] = useState(false);

  // Local state
  const [localPath, setLocalPath] = useState('/');
  const [localEntries, setLocalEntries] = useState<SftpEntry[]>([]);
  const [localLoading, setLocalLoading] = useState(false);
  const [localError, setLocalError] = useState<string | null>(null);

  // Context menu
  const [ctxMenu, setCtxMenu] = useState<ContextMenuState>(initialCtx);
  const [ctxSide, setCtxSide] = useState<PanelSide>('remote');

  // Rename / mkdir modals
  const [renameTarget, setRenameTarget] = useState<{ side: PanelSide; entry: SftpEntry } | null>(null);
  const [renameValue, setRenameValue] = useState('');
  const [mkdirTarget, setMkdirTarget] = useState<PanelSide | null>(null);
  const [mkdirValue, setMkdirValue] = useState('');
  const [chmodTarget, setChmodTarget] = useState<SftpEntry | null>(null);
  const [chmodValue, setChmodValue] = useState('');
  const [editTarget, setEditTarget] = useState<SftpEntry | null>(null);
  const [editValue, setEditValue] = useState('');
  const [editLoading, setEditLoading] = useState(false);

  // Loading overlay for upload/download
  const [transferLoading, setTransferLoading] = useState(false);

  const fileInputRef = useRef<HTMLInputElement>(null);
  const [uploadSide, setUploadSide] = useState<PanelSide>('remote');
  const [dragOverSide, setDragOverSide] = useState<PanelSide | null>(null);

  useEffect(() => {
    if (sessionId || !activeTab?.connectionId || connectStartedRef.current) return;

    connectStartedRef.current = true;
    setSessionLoading(true);
    setRemoteError(null);

    connectTerminal(activeTab.connectionId)
      .then((sid) => {
        updateTab(activeTab.id, { sessionId: sid });
      })
      .catch((e) => {
        connectStartedRef.current = false;
        setRemoteError(e instanceof Error ? e.message : String(e));
      })
      .finally(() => setSessionLoading(false));
  }, [activeTab?.connectionId, activeTab?.id, sessionId, updateTab]);

  // ---- remote helpers ----

  const loadRemote = useCallback(
    async (p: string) => {
      if (!sessionId) return;
      setRemoteLoading(true);
      setRemoteError(null);
      try {
        const list = await sftpListDir(sessionId, p);
        setRemoteEntries(sortEntries(list));
        setRemotePath(p);
      } catch (e: any) {
        setRemoteError(e?.toString?.() ?? 'Failed to list directory');
      } finally {
        setRemoteLoading(false);
      }
    },
    [sessionId],
  );

  // ---- local helpers (via Tauri invoke) ----

  const loadLocal = useCallback(async (p: string) => {
    setLocalLoading(true);
    setLocalError(null);
    try {
      const list = await listLocalDir(p);
      setLocalEntries(sortEntries(list));
      setLocalPath(p);
    } catch (e: any) {
      setLocalError(e?.toString?.() ?? 'Failed to list local directory');
      setLocalEntries([]);
      setLocalPath(p);
    } finally {
      setLocalLoading(false);
    }
  }, []);

  // Initial load
  useEffect(() => {
    loadRemote('/');
  }, [loadRemote]);

  useEffect(() => {
    loadLocal('/');
  }, [loadLocal]);

  // Close context menu on outside click
  useEffect(() => {
    if (!ctxMenu.visible) return;
    const close = () => setCtxMenu(initialCtx);
    window.addEventListener('click', close);
    return () => window.removeEventListener('click', close);
  }, [ctxMenu.visible]);

  // ---- navigation ----

  const navigateRemote = useCallback(
    (entry: SftpEntry) => {
      if (entry.is_dir) {
        loadRemote(entry.path);
      }
    },
    [loadRemote],
  );

  const navigateLocal = useCallback(
    (entry: SftpEntry) => {
      if (entry.is_dir) {
        loadLocal(entry.path);
      }
    },
    [loadLocal],
  );

  const goUpRemote = () => {
    const parent = remotePath.replace(/\/[^/]*\/?$/, '') || '/';
    loadRemote(parent);
  };

  const goUpLocal = () => {
    const parent = localPath.replace(/\/[^/]*\/?$/, '') || '/';
    loadLocal(parent);
  };

  // ---- context menu actions ----

  const openCtx = (e: React.MouseEvent, side: PanelSide, entry: SftpEntry | null) => {
    e.preventDefault();
    e.stopPropagation();
    setCtxSide(side);
    setCtxMenu({ visible: true, x: e.clientX, y: e.clientY, entry });
  };

  const handleDownload = async () => {
    const entry = ctxMenu.entry;
    if (!entry || entry.is_dir || ctxSide !== 'remote') return;
    setCtxMenu(initialCtx);
    setTransferLoading(true);
    try {
      const data = await sftpReadFile(sessionId, entry.path);
      const uint8 = new Uint8Array(data);
      const blob = new Blob([uint8]);
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = entry.name;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e: any) {
      alert('Download failed: ' + (e?.toString?.() ?? e));
    } finally {
      setTransferLoading(false);
    }
  };

  const handleUpload = (side: PanelSide) => {
    setUploadSide(side);
    setCtxMenu(initialCtx);
    fileInputRef.current?.click();
  };

  const uploadFiles = async (files: File[], side: PanelSide) => {
    if (files.length === 0) return;
    if (side === 'remote' && !sessionId) {
      alert('No SSH session available.');
      return;
    }

    setTransferLoading(true);
    try {
      for (const file of files) {
        const buffer = await file.arrayBuffer();
        const data = Array.from(new Uint8Array(buffer));

        if (side === 'remote') {
          await sftpWriteFile(sessionId, joinPath(remotePath, file.name), data);
        } else {
          await writeLocalFile(joinPath(localPath, file.name), data);
        }
      }

      if (side === 'remote') {
        await loadRemote(remotePath);
      } else {
        await loadLocal(localPath);
      }
    } catch (err: any) {
      alert('Upload failed: ' + (err?.toString?.() ?? err));
    } finally {
      setTransferLoading(false);
    }
  };

  const onFileSelected = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files || []);
    e.target.value = ''; // reset
    await uploadFiles(files, uploadSide);
  };

  const filesFromDrop = (e: React.DragEvent<HTMLDivElement>): File[] => {
    const files: File[] = [];
    for (const item of Array.from(e.dataTransfer.items || [])) {
      if (item.kind !== 'file') continue;
      const file = item.getAsFile();
      if (file) files.push(file);
    }
    if (files.length === 0) {
      files.push(...Array.from(e.dataTransfer.files || []));
    }
    return files;
  };

  const handleDragOver = (e: React.DragEvent<HTMLDivElement>, side: PanelSide) => {
    if (!Array.from(e.dataTransfer.types).includes('Files')) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
    setDragOverSide(side);
  };

  const handleDragLeave = (e: React.DragEvent<HTMLDivElement>, side: PanelSide) => {
    const next = e.relatedTarget;
    if (next instanceof Node && e.currentTarget.contains(next)) return;
    if (dragOverSide === side) setDragOverSide(null);
  };

  const handleDrop = async (e: React.DragEvent<HTMLDivElement>, side: PanelSide) => {
    e.preventDefault();
    setDragOverSide(null);
    const files = filesFromDrop(e);
    if (files.length === 0) {
      alert('Only files can be uploaded here.');
      return;
    }
    await uploadFiles(files, side);
  };

  const handleDelete = async () => {
    const entry = ctxMenu.entry;
    if (!entry) return;
    setCtxMenu(initialCtx);
    

    try {
      if (ctxSide === 'remote') {
        await sftpRemoveFile(sessionId, entry.path);
        await loadRemote(remotePath);
      } else {
        await removeLocalFile(entry.path);
        await loadLocal(localPath);
      }
    } catch (e: any) {
      alert('Delete failed: ' + (e?.toString?.() ?? e));
    }
  };

  const startRename = () => {
    const entry = ctxMenu.entry;
    if (!entry) return;
    setCtxMenu(initialCtx);
    setRenameTarget({ side: ctxSide, entry });
    setRenameValue(entry.name);
  };

  const confirmRename = async () => {
    if (!renameTarget || !renameValue.trim()) return;
    const { side, entry } = renameTarget;
    const dir = side === 'remote' ? remotePath : localPath;
    const parent = dir.replace(/\/+$/, '');
    const newPath = `${parent}/${renameValue.trim()}`;

    try {
      if (side === 'remote') {
        await sftpRename(sessionId, entry.path, newPath);
        await loadRemote(remotePath);
      } else {
        await renameLocalFile(entry.path, newPath);
        await loadLocal(localPath);
      }
    } catch (e: any) {
      alert('Rename failed: ' + (e?.toString?.() ?? e));
    }
    setRenameTarget(null);
  };

  const startMkdir = (side: PanelSide) => {
    setCtxMenu(initialCtx);
    setMkdirTarget(side);
    setMkdirValue('');
  };

  const confirmMkdir = async () => {
    if (!mkdirTarget || !mkdirValue.trim()) return;
    const dir = mkdirTarget === 'remote' ? remotePath : localPath;
    const parent = dir.replace(/\/+$/, '');
    const newPath = `${parent}/${mkdirValue.trim()}`;

    try {
      if (mkdirTarget === 'remote') {
        await sftpMkdir(sessionId, newPath);
        await loadRemote(remotePath);
      } else {
        await createLocalDir(newPath);
        await loadLocal(localPath);
      }
    } catch (e: any) {
      alert('Mkdir failed: ' + (e?.toString?.() ?? e));
    }
    setMkdirTarget(null);
  };

  const startChmod = () => {
    const entry = ctxMenu.entry;
    if (!entry || ctxSide !== 'remote') return;
    setCtxMenu(initialCtx);
    setChmodTarget(entry);
    setChmodValue(entry.permissions.padStart(3, '0').slice(-4));
  };

  const confirmChmod = async () => {
    if (!chmodTarget) return;
    const mode = chmodValue.trim();
    if (!/^[0-7]{3,4}$/.test(mode)) {
      alert('Mode must be a 3 or 4 digit octal value, for example 644 or 0755.');
      return;
    }

    try {
      await sftpChmod(sessionId, chmodTarget.path, mode);
      await loadRemote(remotePath);
      setChmodTarget(null);
    } catch (e: any) {
      alert('Chmod failed: ' + (e?.toString?.() ?? e));
    }
  };

  const startEditRemoteFile = async () => {
    const entry = ctxMenu.entry;
    if (!entry || entry.is_dir || ctxSide !== 'remote') return;
    setCtxMenu(initialCtx);

    if (entry.size > MAX_TEXT_EDIT_BYTES) {
      alert('Online editing is limited to text files up to 1 MB.');
      return;
    }

    setEditLoading(true);
    try {
      const data = await sftpReadFile(sessionId, entry.path);
      const bytes = new Uint8Array(data);
      if (bytes.includes(0)) {
        alert('This file looks binary and cannot be edited inline.');
        return;
      }

      const text = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
      setEditTarget(entry);
      setEditValue(text);
    } catch (e: any) {
      alert('Open failed: ' + (e?.toString?.() ?? e));
    } finally {
      setEditLoading(false);
    }
  };

  const confirmEditRemoteFile = async () => {
    if (!editTarget) return;
    setEditLoading(true);
    try {
      const bytes = Array.from(new TextEncoder().encode(editValue));
      await sftpWriteFile(sessionId, editTarget.path, bytes);
      await loadRemote(remotePath);
      setEditTarget(null);
    } catch (e: any) {
      alert('Save failed: ' + (e?.toString?.() ?? e));
    } finally {
      setEditLoading(false);
    }
  };

  // ---- render helpers ----

  const Breadcrumb = ({
    path,
    onNavigate,
  }: {
    path: string;
    onNavigate: (p: string) => void;
  }) => (
    <div className="flex items-center gap-1 px-2 py-1 overflow-x-auto text-xs select-none">
      {pathSegments(path).map((seg, i) => (
        <span key={seg.path} className="flex items-center gap-1 shrink-0">
          {i > 0 && <span className="text-[var(--text-muted)]">/</span>}
          <button
            onClick={() => onNavigate(seg.path)}
            className="hover:text-[var(--accent)] transition-colors px-1 py-0.5 rounded hover:bg-[var(--bg-surface)]"
            style={{ color: 'var(--text-secondary)' }}
          >
            {seg.label}
          </button>
        </span>
      ))}
    </div>
  );

  const FilePanel = ({
    side,
    title,
    icon: Icon,
    entries,
    loading,
    error,
    currentPath,
    onNavigate,
    onGoUp,
    onRefresh,
  }: {
    side: PanelSide;
    title: string;
    icon: typeof HardDrive;
    entries: SftpEntry[];
    loading: boolean;
    error: string | null;
    currentPath: string;
    onNavigate: (entry: SftpEntry) => void;
    onGoUp: () => void;
    onRefresh: () => void;
  }) => (
    <div
      className={`flex flex-col flex-1 min-w-0 border rounded-lg overflow-hidden transition-colors ${
        dragOverSide === side ? 'border-[var(--accent)] bg-[var(--bg-surface)]' : 'border-[var(--border)]'
      }`}
      onDragOver={(e) => handleDragOver(e, side)}
      onDragLeave={(e) => handleDragLeave(e, side)}
      onDrop={(e) => handleDrop(e, side)}
    >
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-1.5 bg-[var(--bg-secondary)] border-b border-[var(--border)]">
        <div className="flex items-center gap-2 text-sm font-medium">
          <Icon size={14} className="text-[var(--accent)]" />
          <span>{title}</span>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={onGoUp}
            className="btn btn-ghost p-1"
            title="Go up"
          >
            <ArrowUp size={14} />
          </button>
          <button
            onClick={() => handleUpload(side)}
            className="btn btn-ghost p-1"
            title="Upload files"
          >
            <Upload size={14} />
          </button>
          <button
            onClick={() => startMkdir(side)}
            className="btn btn-ghost p-1"
            title="New folder"
          >
            <Plus size={14} />
          </button>
          <button
            onClick={onRefresh}
            className="btn btn-ghost p-1"
            title="Refresh"
          >
            <Loader2 size={14} className={loading ? 'animate-spin' : ''} />
          </button>
        </div>
      </div>

      {/* Breadcrumb */}
      <div className="bg-[var(--bg-secondary)] border-b border-[var(--border)]">
        <Breadcrumb path={currentPath} onNavigate={(p) => { side === 'remote' ? loadRemote(p) : loadLocal(p); }} />
      </div>

      {/* File list */}
      <div
        className="flex-1 overflow-y-auto relative"
        onContextMenu={(e) => openCtx(e, side, null)}
      >
        {loading && (
          <div className="absolute inset-0 flex items-center justify-center bg-[var(--bg-primary)]/60 z-10">
            <Loader2 size={20} className="animate-spin text-[var(--accent)]" />
          </div>
        )}
        {error && (
          <div className="px-3 py-2 text-xs text-[var(--error)]">{error}</div>
        )}
        {dragOverSide === side && (
          <div className="absolute inset-0 z-20 flex items-center justify-center bg-[var(--bg-primary)]/70 pointer-events-none">
            <div className="rounded-md border border-[var(--accent)] bg-[var(--bg-surface)] px-4 py-3 text-sm text-[var(--text-primary)]">
              Drop files to upload
            </div>
          </div>
        )}
        {/* Table header */}
        <div className="grid gap-2 px-3 py-1 text-xs text-[var(--text-muted)] border-b border-[var(--border)] bg-[var(--bg-secondary)] sticky top-0"
          style={{ gridTemplateColumns: '1fr 80px 100px 130px' }}
        >
          <span>Name</span>
          <span className="text-right">Size</span>
          <span>Permissions</span>
          <span>Modified</span>
        </div>
        {/* Rows */}
        {entries.map((entry) => (
          <div
            key={entry.path}
            className="grid gap-2 px-3 py-1 text-xs cursor-pointer hover:bg-[var(--bg-surface)] transition-colors items-center"
            style={{ gridTemplateColumns: '1fr 80px 100px 130px' }}
            onDoubleClick={() => onNavigate(entry)}
            onContextMenu={(e) => openCtx(e, side, entry)}
          >
            <span className="flex items-center gap-2 min-w-0">
              {entry.is_dir ? (
                <Folder size={14} className="text-[var(--warning)] shrink-0" />
              ) : (
                <File size={14} className="text-[var(--text-muted)] shrink-0" />
              )}
              <span className="truncate">{entry.name}</span>
            </span>
            <span className="text-right text-[var(--text-secondary)]">
              {entry.is_dir ? '-' : formatSize(entry.size)}
            </span>
            <span className="text-[var(--text-secondary)] font-mono text-[11px]">
              {formatPermissions(entry.permissions)}
            </span>
            <span className="text-[var(--text-secondary)]">
              {formatDate(entry.modified)}
            </span>
          </div>
        ))}
        {!loading && entries.length === 0 && !error && (
          <div className="px-3 py-6 text-center text-xs text-[var(--text-muted)]">
            Empty directory
          </div>
        )}
      </div>

      {/* Status bar */}
      <div className="status-bar">
        <span>{entries.length} items</span>
        <span className="truncate ml-2">{currentPath}</span>
      </div>
    </div>
  );

  // ---- main render ----

  return (
    <div className="flex flex-col h-full relative">
      {/* Transfer overlay */}
      {transferLoading && (
        <div className="absolute inset-0 z-50 flex items-center justify-center bg-[var(--bg-primary)]/70 backdrop-blur-sm">
          <div className="flex items-center gap-2 px-4 py-3 rounded-lg bg-[var(--bg-surface)] border border-[var(--border)] shadow-lg">
            <Loader2 size={16} className="animate-spin text-[var(--accent)]" />
            <span className="text-sm">Transferring...</span>
          </div>
        </div>
      )}

      {/* Two panels */}
      <div className="flex gap-2 flex-1 p-2 min-h-0">
        {/* Remote */}
        <FilePanel
          side="remote"
          title="Remote"
          icon={Globe}
          entries={remoteEntries}
          loading={sessionLoading || remoteLoading}
          error={remoteError || (!sessionLoading && !sessionId ? 'No SSH session available' : null)}
          currentPath={remotePath}
          onNavigate={navigateRemote}
          onGoUp={goUpRemote}
          onRefresh={() => loadRemote(remotePath)}
        />

        {/* Divider */}
        <div className="w-1 bg-[var(--border)] rounded shrink-0 self-stretch" />

        {/* Local */}
        <FilePanel
          side="local"
          title="Local"
          icon={HardDrive}
          entries={localEntries}
          loading={localLoading}
          error={localError}
          currentPath={localPath}
          onNavigate={navigateLocal}
          onGoUp={goUpLocal}
          onRefresh={() => loadLocal(localPath)}
        />
      </div>

      {/* Hidden file input for uploads */}
      <input
        ref={fileInputRef}
        type="file"
        multiple
        className="hidden"
        onChange={onFileSelected}
      />

      {/* Context menu */}
      {ctxMenu.visible && (
        <div
          className="context-menu animate-fade-in"
          style={{ left: ctxMenu.x, top: ctxMenu.y }}
          onClick={(e) => e.stopPropagation()}
        >
          {ctxMenu.entry ? (
            <>
              {ctxMenu.entry.is_dir && (
                <div
                  className="context-menu-item"
                  onClick={() => {
                    setCtxMenu(initialCtx);
                    ctxSide === 'remote'
                      ? navigateRemote(ctxMenu.entry!)
                      : navigateLocal(ctxMenu.entry!);
                  }}
                >
                  <Folder size={14} />
                  <span>Open</span>
                </div>
              )}
              {!ctxMenu.entry.is_dir && ctxSide === 'remote' && (
                <div className="context-menu-item" onClick={handleDownload}>
                  <Download size={14} />
                  <span>Download</span>
                </div>
              )}
              {!ctxMenu.entry.is_dir && ctxSide === 'remote' && (
                <div className="context-menu-item" onClick={startEditRemoteFile}>
                  <Edit size={14} />
                  <span>Edit</span>
                </div>
              )}
              <div className="context-menu-item" onClick={startRename}>
                <Edit size={14} />
                <span>Rename</span>
              </div>
              {ctxSide === 'remote' && (
                <div className="context-menu-item" onClick={startChmod}>
                  <Edit size={14} />
                  <span>Permissions</span>
                </div>
              )}
              <div className="context-menu-divider" />
              <div
                className="context-menu-item text-[var(--error)]"
                onClick={handleDelete}
              >
                <Trash2 size={14} />
                <span>Delete</span>
              </div>
            </>
          ) : (
            <>
              <div
                className="context-menu-item"
                onClick={() => handleUpload(ctxSide)}
              >
                <Upload size={14} />
                <span>Upload files</span>
              </div>
              <div
                className="context-menu-item"
                onClick={() => startMkdir(ctxSide)}
              >
                <Plus size={14} />
                <span>New folder</span>
              </div>
            </>
          )}
        </div>
      )}

      {/* Rename modal */}
      {renameTarget && (
        <div className="modal-overlay" onClick={() => setRenameTarget(null)}>
          <div className="modal animate-slide-in" onClick={(e) => e.stopPropagation()}>
            <div className="modal-title">Rename</div>
            <input
              className="input mb-4"
              value={renameValue}
              onChange={(e) => setRenameValue(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && confirmRename()}
              autoFocus
            />
            <div className="flex justify-end gap-2">
              <button className="btn btn-secondary" onClick={() => setRenameTarget(null)}>
                Cancel
              </button>
              <button className="btn btn-primary" onClick={confirmRename}>
                Rename
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Mkdir modal */}
      {mkdirTarget !== null && (
        <div className="modal-overlay" onClick={() => setMkdirTarget(null)}>
          <div className="modal animate-slide-in" onClick={(e) => e.stopPropagation()}>
            <div className="modal-title">New Folder</div>
            <input
              className="input mb-4"
              placeholder="Folder name"
              value={mkdirValue}
              onChange={(e) => setMkdirValue(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && confirmMkdir()}
              autoFocus
            />
            <div className="flex justify-end gap-2">
              <button className="btn btn-secondary" onClick={() => setMkdirTarget(null)}>
                Cancel
              </button>
              <button className="btn btn-primary" onClick={confirmMkdir}>
                Create
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Chmod modal */}
      {chmodTarget && (
        <div className="modal-overlay" onClick={() => setChmodTarget(null)}>
          <div className="modal animate-slide-in" onClick={(e) => e.stopPropagation()}>
            <div className="modal-title">Permissions</div>
            <div className="text-xs mb-2 text-[var(--text-secondary)] truncate">
              {chmodTarget.path}
            </div>
            <input
              className="input mb-4 font-mono"
              placeholder="755"
              value={chmodValue}
              onChange={(e) => setChmodValue(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && confirmChmod()}
              autoFocus
            />
            <div className="flex justify-end gap-2">
              <button className="btn btn-secondary" onClick={() => setChmodTarget(null)}>
                Cancel
              </button>
              <button className="btn btn-primary" onClick={confirmChmod}>
                Apply
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Remote text edit modal */}
      {editTarget && (
        <div className="modal-overlay" onClick={() => setEditTarget(null)}>
          <div
            className="modal animate-slide-in flex flex-col"
            style={{ width: 'min(860px, 92vw)', height: 'min(680px, 86vh)' }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-title">Edit Remote File</div>
            <div className="text-xs mb-2 text-[var(--text-secondary)] truncate">
              {editTarget.path}
            </div>
            <textarea
              className="input flex-1 min-h-0 resize-none font-mono text-xs leading-5"
              value={editValue}
              onChange={(e) => setEditValue(e.target.value)}
              spellCheck={false}
              autoFocus
            />
            <div className="flex justify-end gap-2 mt-4">
              <button className="btn btn-secondary" onClick={() => setEditTarget(null)}>
                Cancel
              </button>
              <button className="btn btn-primary" onClick={confirmEditRemoteFile} disabled={editLoading}>
                {editLoading ? 'Saving...' : 'Save'}
              </button>
            </div>
          </div>
        </div>
      )}

      {editLoading && !editTarget && (
        <div className="absolute inset-0 z-50 flex items-center justify-center bg-[var(--bg-primary)]/70 backdrop-blur-sm">
          <div className="flex items-center gap-2 px-4 py-3 rounded-lg bg-[var(--bg-surface)] border border-[var(--border)] shadow-lg">
            <Loader2 size={16} className="animate-spin text-[var(--accent)]" />
            <span className="text-sm">Opening...</span>
          </div>
        </div>
      )}
    </div>
  );
}
