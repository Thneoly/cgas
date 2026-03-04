import { ChangeEvent, useEffect, useMemo, useRef, useState } from 'react';

type ConversationItem = {
  round: number;
  from: string;
  to: string;
  decision: string;
  summary: string;
  time?: string;
  timestamp?: string;
};

type BlackboardEvent = {
  id: number;
  round: number;
  from: string;
  event_type: string;
  decision: string;
  summary: string;
  next_role?: string | null;
  audit?: Record<string, unknown>;
};

type BlackboardState = {
  version: number;
  slots: Record<string, unknown>;
  events: BlackboardEvent[];
};

type SelectedItem = {
  title: string;
  subtitle: string;
  payload: unknown;
};

const ROLE_ORDER = ['PM', 'Dev', 'QA', 'Security', 'SRE'];

const getDataBase = () => {
  const q = new URLSearchParams(window.location.search);
  return q.get('data') ?? '/week1';
};

function parseJsonText<T>(text: string, source: string): T {
  try {
    return JSON.parse(text) as T;
  } catch {
    const preview = text.slice(0, 60).replace(/\s+/g, ' ').trim();
    if (preview.startsWith('<')) {
      throw new Error(
        `Invalid JSON from ${source}. Server returned HTML (check data path).`
      );
    }
    throw new Error(`Invalid JSON from ${source}. Response preview: ${preview}`);
  }
}

async function loadJson<T>(path: string): Promise<T> {
  const res = await fetch(path);
  if (!res.ok) {
    throw new Error(`Failed to load ${path}: ${res.status}`);
  }
  const text = await res.text();
  return parseJsonText<T>(text, path);
}

export default function App() {
  const [conversation, setConversation] = useState<ConversationItem[]>([]);
  const [blackboard, setBlackboard] = useState<BlackboardState | null>(null);
  const [selectedRole, setSelectedRole] = useState<string>('ALL');
  const [selectedItem, setSelectedItem] = useState<SelectedItem | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [dataSource, setDataSource] = useState<string>('URL: /week1');
  const fileInputRef = useRef<HTMLInputElement>(null);
  const directoryInputProps = { webkitdirectory: '', directory: '' } as any;

  const applyLoadedData = (
    collaboration: ConversationItem[],
    board: BlackboardState,
    sourceLabel: string
  ) => {
    setConversation(collaboration);
    setBlackboard(board);
    setDataSource(sourceLabel);

    const first = collaboration[0];
    if (first) {
      setSelectedItem({
        title: `Round ${first.round}: ${first.from} → ${first.to}`,
        subtitle: `decision=${first.decision}`,
        payload: first,
      });
    } else {
      setSelectedItem(null);
    }
  };

  const loadFromBase = (base: string) => {
    setLoading(true);
    setError(null);

    Promise.all([
      loadJson<ConversationItem[]>(`${base}/collaboration_log.json`),
      loadJson<BlackboardState>(`${base}/blackboard_state.json`),
    ])
      .then(([collaboration, board]) => {
        applyLoadedData(collaboration, board, `URL: ${base}`);
      })
      .catch((e: unknown) => {
        setError(e instanceof Error ? e.message : 'Unknown error');
      })
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    loadFromBase(getDataBase());
  }, []);

  const pickFolderViaBrowser = async () => {
    const picker = (
      window as Window & { showDirectoryPicker?: () => Promise<any> }
    ).showDirectoryPicker;

    if (!picker) {
      fileInputRef.current?.click();
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const dirHandle = await picker();
      const [collabHandle, boardHandle] = await Promise.all([
        dirHandle.getFileHandle('collaboration_log.json'),
        dirHandle.getFileHandle('blackboard_state.json'),
      ]);

      const [collabText, boardText] = await Promise.all([
        (await collabHandle.getFile()).text(),
        (await boardHandle.getFile()).text(),
      ]);

      const collaboration = parseJsonText<ConversationItem[]>(
        collabText,
        'selected-folder/collaboration_log.json'
      );
      const board = parseJsonText<BlackboardState>(
        boardText,
        'selected-folder/blackboard_state.json'
      );
      applyLoadedData(collaboration, board, 'Folder: selected via picker');
    } catch (e: unknown) {
      const message = e instanceof Error ? e.message : 'Unknown error';
      if (!message.toLowerCase().includes('abort')) {
        setError(message);
      }
    } finally {
      setLoading(false);
    }
  };

  const handleFolderInput = async (event: ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(event.target.files ?? []);
    if (files.length === 0) {
      return;
    }

    const collabFile = files.find((file) => file.name === 'collaboration_log.json');
    const boardFile = files.find((file) => file.name === 'blackboard_state.json');
    if (!collabFile || !boardFile) {
      setError('Selected folder must contain collaboration_log.json and blackboard_state.json');
      event.target.value = '';
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const [collabText, boardText] = await Promise.all([
        collabFile.text(),
        boardFile.text(),
      ]);
      const collaboration = parseJsonText<ConversationItem[]>(
        collabText,
        collabFile.name
      );
      const board = parseJsonText<BlackboardState>(boardText, boardFile.name);
      const rootDir = collabFile.webkitRelativePath.split('/')[0] || 'selected-folder';
      applyLoadedData(collaboration, board, `Folder: ${rootDir}`);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : 'Unknown error');
    } finally {
      event.target.value = '';
      setLoading(false);
    }
  };

  const filteredConversation = useMemo(() => {
    if (selectedRole === 'ALL') {
      return conversation;
    }
    return conversation.filter(
      (item) => item.from === selectedRole || item.to === selectedRole
    );
  }, [conversation, selectedRole]);

  const conversationMessages = useMemo(
    () =>
      filteredConversation.map((item, idx) => {
        const rawTime = item.time ?? item.timestamp;
        const displayTime =
          typeof rawTime === 'string' && rawTime.trim().length > 0
            ? rawTime
            : `T+${idx * 3}m`;

        return {
          ...item,
          id: `${item.round}-${item.from}-${item.to}-${idx}`,
          mention: `@${item.to}`,
          displayTime,
          side: idx % 2 === 0 ? 'left' : 'right',
        };
      }),
    [filteredConversation]
  );

  const roleStats = useMemo(() => {
    const stats: Record<string, number> = {};
    for (const row of conversation) {
      stats[row.from] = (stats[row.from] ?? 0) + 1;
    }
    return stats;
  }, [conversation]);

  const dispatchEvents = useMemo(
    () =>
      (blackboard?.events ?? []).filter(
        (event) => event.event_type === 'dispatch_decided'
      ),
    [blackboard]
  );

  if (loading) {
    return <div className="center">Loading report...</div>;
  }

  if (error || !blackboard) {
    return (
      <div className="center error">
        <h2>Failed to load report</h2>
        <p>{error ?? 'blackboard_state.json missing'}</p>
        <p>Tip: use /week1, append ?data=/week1, or click Load Folder.</p>
      </div>
    );
  }

  return (
    <div className="page">
      <header className="header">
        <div>
          <h1>Workflow Offline Report</h1>
          <p>Playwright-style local report for phase workflow conversations</p>
        </div>
        <div className="meta">
          <span>{dataSource}</span>
          <span>Blackboard v{blackboard.version}</span>
          <span>Events {blackboard.events.length}</span>
          <span>Dispatch {dispatchEvents.length}</span>
        </div>
      </header>

      <section className="stats">
        <button className="chip" onClick={() => loadFromBase(getDataBase())}>
          Reload URL Data
        </button>
        <button className="chip" onClick={pickFolderViaBrowser}>
          Load Folder
        </button>
        <input
          {...directoryInputProps}
          ref={fileInputRef}
          type="file"
          multiple
          onChange={handleFolderInput}
          style={{ display: 'none' }}
        />

        <button
          className={selectedRole === 'ALL' ? 'chip active' : 'chip'}
          onClick={() => setSelectedRole('ALL')}
        >
          ALL ({conversation.length})
        </button>
        {ROLE_ORDER.map((role) => (
          <button
            key={role}
            className={selectedRole === role ? 'chip active' : 'chip'}
            onClick={() => setSelectedRole(role)}
          >
            {role} ({roleStats[role] ?? 0})
          </button>
        ))}
      </section>

      <main className="layout">
        <aside className="panel list">
          <div className="panel-title">Conversation Timeline</div>
          <div className="chat-list">
            {conversationMessages.map((item) => (
            <button
              key={item.id}
              className={`chat-message ${item.side}`}
              onClick={() =>
                setSelectedItem({
                  title: `Round ${item.round}: ${item.from} → ${item.to}`,
                  subtitle: `decision=${item.decision}`,
                  payload: item,
                })
              }
            >
              <div className="chat-avatar">{item.from.slice(0, 1)}</div>
              <div className="chat-body">
                <div className="chat-meta">
                  <span className="chat-speaker">{item.from}</span>
                  <span className="chat-time">R{item.round} · {item.displayTime}</span>
                </div>
                <div className="chat-bubble">
                  <div className="chat-content">{item.summary}</div>
                  <div className="chat-footer">
                    <span className="chat-mention">{item.mention}</span>
                    <span
                      className={`decision-badge ${
                        item.decision === 'approved' ? 'ok' : 'bad'
                      }`}
                    >
                      {item.decision}
                    </span>
                  </div>
                </div>
              </div>
            </button>
            ))}
          </div>
        </aside>

        <section className="panel detail">
          <div className="panel-title">Details</div>
          {selectedItem ? (
            <>
              <h3>{selectedItem.title}</h3>
              <p className="subtitle">{selectedItem.subtitle}</p>
              <pre>{JSON.stringify(selectedItem.payload, null, 2)}</pre>
            </>
          ) : (
            <p>Select one timeline item to inspect details.</p>
          )}
        </section>

        <aside className="panel list">
          <div className="panel-title">Dispatch Audit</div>
          {dispatchEvents.map((event) => (
            <button
              key={event.id}
              className="timeline-item"
              onClick={() =>
                setSelectedItem({
                  title: `Dispatch #${event.id}: ${event.from} -> ${event.next_role ?? '-'}`,
                  subtitle: `round=${event.round}`,
                  payload: event,
                })
              }
            >
              <div className="timeline-head">
                <span>#{event.id}</span>
                <span>{`${event.from} -> ${event.next_role ?? '-'}`}</span>
              </div>
              <div className="timeline-summary">
                {(event.audit?.rule_id as string | undefined) ?? 'no rule_id'}
              </div>
            </button>
          ))}
        </aside>
      </main>
    </div>
  );
}
