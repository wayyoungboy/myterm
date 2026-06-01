import { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { Plus, Trash2, FileText, Search, Save } from 'lucide-react';
import { useAppStore } from '../../stores/appStore';
import { getNotes, createNote, updateNote, deleteNote } from '../../utils/tauri';
import type { Note } from '../../types';

// ── Helpers ──────────────────────────────────────────────────────────

function formatDate(iso: string | null): string {
  if (!iso) return '';
  try {
    const d = new Date(iso);
    if (isNaN(d.getTime())) return '';
    const now = new Date();
    const isToday =
      d.getFullYear() === now.getFullYear() &&
      d.getMonth() === now.getMonth() &&
      d.getDate() === now.getDate();

    const pad = (n: number) => String(n).padStart(2, '0');
    if (isToday) {
      return `Today ${pad(d.getHours())}:${pad(d.getMinutes())}`;
    }
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
  } catch {
    return '';
  }
}

function contentPreview(content: string | null, maxLen = 80): string {
  if (!content) return 'Empty note';
  const stripped = content.replace(/[#*_`~\[\]()]/g, '').trim();
  if (stripped.length <= maxLen) return stripped;
  return stripped.slice(0, maxLen) + '...';
}

// ── Component ────────────────────────────────────────────────────────

export function NotesView() {
  const { tabs, activeTabId, selectedConnectionId } = useAppStore();
  const activeTab = tabs.find((t) => t.id === activeTabId);
  const connectionId = activeTab?.connectionId ?? selectedConnectionId ?? null;

  const [notes, setNotes] = useState<Note[]>([]);
  const [selectedNoteId, setSelectedNoteId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Editor state
  const [editTitle, setEditTitle] = useState('');
  const [editContent, setEditContent] = useState('');
  const [saving, setSaving] = useState(false);
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);

  // Search and filter
  const [searchQuery, setSearchQuery] = useState('');
  const [filterMode, setFilterMode] = useState<'connection' | 'all'>('connection');

  // Debounce timer ref
  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isEditingRef = useRef(false);
  const editTitleRef = useRef('');
  const editContentRef = useRef('');

  // ── Data loading ─────────────────────────────────────────────────

  const loadNotes = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const filterId = filterMode === 'connection' ? connectionId : undefined;
      const result = await getNotes(filterId ?? undefined);
      setNotes(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load notes');
    } finally {
      setLoading(false);
    }
  }, [connectionId, filterMode]);

  useEffect(() => {
    loadNotes();
  }, [loadNotes]);

  // ── Filtered notes ───────────────────────────────────────────────

  const filteredNotes = useMemo(() => {
    if (!searchQuery.trim()) return notes;
    const q = searchQuery.toLowerCase();
    return notes.filter(
      (n) =>
        (n.title ?? '').toLowerCase().includes(q) ||
        (n.content ?? '').toLowerCase().includes(q),
    );
  }, [notes, searchQuery]);

  // ── Select note ──────────────────────────────────────────────────

  const selectedNote = useMemo(
    () => notes.find((n) => n.id === selectedNoteId) ?? null,
    [notes, selectedNoteId],
  );

  useEffect(() => {
    if (selectedNote) {
      isEditingRef.current = true;
      setEditTitle(selectedNote.title ?? '');
      setEditContent(selectedNote.content ?? '');
      setHasUnsavedChanges(false);
      isEditingRef.current = false;
    } else {
      setEditTitle('');
      setEditContent('');
      setHasUnsavedChanges(false);
    }
  }, [selectedNoteId]); // eslint-disable-line react-hooks/exhaustive-deps

  // ── Auto-save (debounced 1s) ─────────────────────────────────────

  const scheduleSave = useCallback(
    () => {
      if (!selectedNoteId) return;
      setHasUnsavedChanges(true);

      if (saveTimerRef.current) {
        clearTimeout(saveTimerRef.current);
      }

      saveTimerRef.current = setTimeout(async () => {
        setSaving(true);
        try {
          // Read current values from refs to avoid stale closure
          await updateNote(selectedNoteId, editTitleRef.current, editContentRef.current);
          // Refresh notes list to reflect updated content
          const filterId = filterMode === 'connection' ? connectionId : undefined;
          const result = await getNotes(filterId ?? undefined);
          setNotes(result);
          setHasUnsavedChanges(false);
        } catch (e) {
          setError(e instanceof Error ? e.message : 'Auto-save failed');
        } finally {
          setSaving(false);
        }
      }, 1000);
    },
    [selectedNoteId, connectionId, filterMode],
  );

  // Clean up timer on unmount
  useEffect(() => {
    return () => {
      if (saveTimerRef.current) {
        clearTimeout(saveTimerRef.current);
      }
    };
  }, []);

  const handleTitleChange = (value: string) => {
    setEditTitle(value);
    editTitleRef.current = value;
    if (!isEditingRef.current) {
      scheduleSave();
    }
  };

  const handleContentChange = (value: string) => {
    setEditContent(value);
    editContentRef.current = value;
    if (!isEditingRef.current) {
      scheduleSave();
    }
  };

  // ── Manual save ──────────────────────────────────────────────────

  const handleManualSave = async () => {
    if (!selectedNoteId) return;
    if (saveTimerRef.current) {
      clearTimeout(saveTimerRef.current);
      saveTimerRef.current = null;
    }
    setSaving(true);
    try {
      await updateNote(selectedNoteId, editTitle, editContent);
      setHasUnsavedChanges(false);
      const filterId = filterMode === 'connection' ? connectionId : undefined;
      const result = await getNotes(filterId ?? undefined);
      setNotes(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to save note');
    } finally {
      setSaving(false);
    }
  };

  // ── Create note ──────────────────────────────────────────────────

  const handleCreateNote = async () => {
    try {
      const noteConnectionId = filterMode === 'connection' ? connectionId : undefined;
      const newNote = await createNote('Untitled', '', noteConnectionId ?? undefined);
      const filterId = filterMode === 'connection' ? connectionId : undefined;
      const result = await getNotes(filterId ?? undefined);
      setNotes(result);
      setSelectedNoteId(newNote.id);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create note');
    }
  };

  // ── Delete note ──────────────────────────────────────────────────

  const handleDeleteNote = async () => {
    if (!selectedNoteId) return;
    if (!confirm('Delete this note?')) return;

    try {
      await deleteNote(selectedNoteId);
      setSelectedNoteId(null);
      const filterId = filterMode === 'connection' ? connectionId : undefined;
      const result = await getNotes(filterId ?? undefined);
      setNotes(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to delete note');
    }
  };

  // ── Keyboard shortcut: Ctrl+S to save ────────────────────────────

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault();
        handleManualSave();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleManualSave]);

  // ── Render ───────────────────────────────────────────────────────

  return (
    <div className="flex h-full w-full" style={{ background: 'var(--bg-primary)' }}>
      {/* ── Left Panel: Notes List ── */}
      <div
        className="flex flex-col border-r shrink-0"
        style={{
          width: 280,
          borderColor: 'var(--border)',
          background: 'var(--bg-secondary)',
        }}
      >
        {/* Toolbar */}
        <div
          className="flex items-center gap-1 px-2 py-1.5 border-b"
          style={{ borderColor: 'var(--border)' }}
        >
          <button
            className="btn btn-ghost p-1"
            onClick={handleCreateNote}
            title="New note"
          >
            <Plus size={14} />
          </button>

          {/* Filter toggle */}
          <button
            className="btn btn-ghost p-1 text-[11px]"
            onClick={() => setFilterMode(filterMode === 'connection' ? 'all' : 'connection')}
            title={filterMode === 'connection' ? 'Showing connection notes' : 'Showing all notes'}
            style={{
              color: filterMode === 'connection' ? 'var(--accent)' : 'var(--text-muted)',
            }}
          >
            <FileText size={14} />
          </button>

          <div className="flex-1" />

          {selectedNoteId && (
            <button
              className="btn btn-ghost p-1"
              onClick={handleDeleteNote}
              title="Delete note"
            >
              <Trash2 size={14} style={{ color: 'var(--error)' }} />
            </button>
          )}
        </div>

        {/* Search */}
        <div className="px-2 py-1.5">
          <div className="relative">
            <Search
              size={12}
              className="absolute left-2 top-1/2 -translate-y-1/2"
              style={{ color: 'var(--text-muted)' }}
            />
            <input
              type="text"
              className="input pl-7 text-xs"
              placeholder="Search notes..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
        </div>

        {/* Filter label */}
        <div
          className="px-3 py-1 text-[10px] uppercase tracking-wide"
          style={{ color: 'var(--text-muted)' }}
        >
          {filterMode === 'connection' && connectionId
            ? 'Connection Notes'
            : 'All Notes'}
        </div>

        {/* Notes list */}
        <div className="flex-1 overflow-y-auto">
          {loading && (
            <div className="px-3 py-4 text-center text-xs" style={{ color: 'var(--text-muted)' }}>
              Loading...
            </div>
          )}

          {error && (
            <div className="px-3 py-2 text-xs" style={{ color: 'var(--error)' }}>
              {error}
            </div>
          )}

          {!loading && filteredNotes.length === 0 && (
            <div className="flex flex-col items-center justify-center py-12 gap-2">
              <FileText size={32} style={{ color: 'var(--text-muted)' }} />
              <span className="text-sm" style={{ color: 'var(--text-muted)' }}>
                No notes yet
              </span>
              <button
                className="btn btn-secondary text-xs mt-1"
                onClick={handleCreateNote}
              >
                <Plus size={12} />
                Create Note
              </button>
            </div>
          )}

          {filteredNotes.map((note) => (
            <div
              key={note.id}
              className="px-3 py-2 cursor-pointer transition-colors border-b"
              style={{
                borderColor: 'var(--border)',
                background:
                  selectedNoteId === note.id ? 'var(--bg-surface)' : 'transparent',
              }}
              onClick={() => setSelectedNoteId(note.id)}
              onMouseEnter={(e) => {
                if (selectedNoteId !== note.id) {
                  (e.currentTarget as HTMLElement).style.background = 'var(--bg-hover)';
                }
              }}
              onMouseLeave={(e) => {
                if (selectedNoteId !== note.id) {
                  (e.currentTarget as HTMLElement).style.background = 'transparent';
                }
              }}
            >
              <div className="flex items-center gap-2">
                <FileText size={12} style={{ color: 'var(--text-muted)', flexShrink: 0 }} />
                <span
                  className="text-xs font-medium truncate"
                  style={{ color: 'var(--text-primary)' }}
                >
                  {note.title || 'Untitled'}
                </span>
              </div>
              <div
                className="text-[11px] mt-0.5 truncate pl-5"
                style={{ color: 'var(--text-muted)' }}
              >
                {contentPreview(note.content)}
              </div>
              <div
                className="text-[10px] mt-0.5 pl-5"
                style={{ color: 'var(--text-muted)' }}
              >
                {formatDate(note.updated_at ?? note.created_at)}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* ── Right Panel: Note Editor ── */}
      <div className="flex flex-col flex-1 min-w-0">
        {selectedNote ? (
          <>
            {/* Editor toolbar */}
            <div
              className="flex items-center gap-2 px-3 py-1.5 border-b"
              style={{ borderColor: 'var(--border)', background: 'var(--bg-secondary)' }}
            >
              <input
                type="text"
                className="input flex-1 text-sm font-medium"
                placeholder="Note title"
                value={editTitle}
                onChange={(e) => handleTitleChange(e.target.value)}
              />

              <div className="flex items-center gap-2 shrink-0">
                {saving && (
                  <span className="text-[10px]" style={{ color: 'var(--text-muted)' }}>
                    Saving...
                  </span>
                )}
                {!saving && hasUnsavedChanges && (
                  <span className="text-[10px]" style={{ color: 'var(--warning)' }}>
                    Unsaved
                  </span>
                )}
                {!saving && !hasUnsavedChanges && selectedNoteId && (
                  <span className="text-[10px]" style={{ color: 'var(--success)' }}>
                    Saved
                  </span>
                )}

                <button
                  className="btn btn-ghost p-1"
                  onClick={handleManualSave}
                  title="Save (Ctrl+S)"
                  disabled={!hasUnsavedChanges}
                >
                  <Save size={14} />
                </button>

                <button
                  className="btn btn-ghost p-1"
                  onClick={handleDeleteNote}
                  title="Delete note"
                >
                  <Trash2 size={14} style={{ color: 'var(--error)' }} />
                </button>
              </div>
            </div>

            {/* Content textarea */}
            <div className="flex-1 min-h-0">
              <textarea
                className="w-full h-full resize-none p-4 text-sm leading-relaxed outline-none"
                style={{
                  background: 'var(--bg-primary)',
                  color: 'var(--text-primary)',
                  fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
                  fontSize: 13,
                }}
                placeholder="Write your Markdown content here..."
                value={editContent}
                onChange={(e) => handleContentChange(e.target.value)}
              />
            </div>

            {/* Status bar */}
            <div
              className="flex items-center justify-between px-3 py-1 text-[10px] border-t"
              style={{
                borderColor: 'var(--border)',
                background: 'var(--bg-secondary)',
                color: 'var(--text-muted)',
              }}
            >
              <span>
                {editContent.length} chars · {editContent.split(/\r?\n/).length} lines
              </span>
              <span>Markdown</span>
            </div>
          </>
        ) : (
          /* Empty state */
          <div className="flex flex-col items-center justify-center h-full gap-3">
            <FileText size={48} style={{ color: 'var(--text-muted)' }} />
            <span className="text-sm" style={{ color: 'var(--text-muted)' }}>
              {notes.length === 0
                ? 'No notes yet. Create one to get started.'
                : 'Select a note from the list'}
            </span>
            {notes.length === 0 && (
              <button className="btn btn-primary text-xs" onClick={handleCreateNote}>
                <Plus size={12} />
                New Note
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

export default NotesView;
