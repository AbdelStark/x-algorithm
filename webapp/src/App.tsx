import { useEffect, useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";

const SNAPSHOT_KEY = "viralitySnapshots";
const LOG_LIMIT = 80;

type MediaType = "none" | "image" | "video" | "gif";

type ActionProbs = {
  like: number;
  reply: number;
  repost: number;
  quote: number;
  click: number;
  profileClick: number;
  videoView: number;
  photoExpand: number;
  share: number;
  dwell: number;
  followAuthor: number;
  notInterested: number;
  block: number;
  mute: number;
  report: number;
};

type Signals = {
  contentQuality: number;
  hook: number;
  authorQuality: number;
  audienceAlignment: number;
  negativeRisk: number;
  shareability: number;
  clarity: number;
  lengthScore: number;
  mediaScore: number;
  timeScore: number;
  novelty: number;
  timeliness: number;
};

type LlmScore = {
  hook: number;
  clarity: number;
  novelty: number;
  shareability: number;
  controversy: number;
  sentiment: number;
  suggestions: string[];
};

type LlmTrace = {
  model: string;
  latency_ms: number;
  prompt_summary: string;
  raw_response: string;
  prompt_tokens?: number | null;
  completion_tokens?: number | null;
  total_tokens?: number | null;
};

type SimulationResult = {
  score: number;
  tier: string;
  weightedScore: number;
  impressionsIn: number;
  impressionsOon: number;
  impressionsTotal: number;
  expectedUniqueEngagements: number;
  expectedActionVolume: number;
  uniqueEngagementRate: number;
  actionVolumeRate: number;
  actions: ActionProbs;
  signals: Signals;
  suggestions: string[];
  warnings: string[];
  llm?: LlmScore;
  llmTrace?: LlmTrace;
  requestId?: string;
};

type ActivityStep = {
  label: string;
  status: "pending" | "active" | "done" | "error";
};

type LogEntry = {
  id: string;
  timestamp: string;
  message: string;
  event: string;
};

type Snapshot = {
  id: string;
  createdAt: string;
  input: FormState;
  output: SimulationResult;
};

type FormState = {
  text: string;
  followers: number;
  following: number;
  accountAgeDays: number;
  avgEngagementRate: number;
  postsPerDay: number;
  hourOfDay: number;
  media: MediaType;
  hasLink: boolean;
  verified: boolean;
  novelty: number;
  timeliness: number;
  audienceFit: number;
  topicSaturation: number;
  controversy: number;
  sentiment: number;
  useAi: boolean;
};

const DEFAULT_FORM: FormState = {
  text: "This is a banger!",
  followers: 1000,
  following: 500,
  accountAgeDays: 365,
  avgEngagementRate: 0.02,
  postsPerDay: 2,
  hourOfDay: 12,
  media: "none",
  hasLink: false,
  verified: false,
  novelty: 0.5,
  timeliness: 0.5,
  audienceFit: 0.6,
  topicSaturation: 0.5,
  controversy: 0.3,
  sentiment: 0.1,
  useAi: false,
};

const INITIAL_ACTIVITY: ActivityStep[] = [
  { label: "Preparing prompt", status: "pending" },
  { label: "Calling Grok API", status: "pending" },
  { label: "Merging signals", status: "pending" },
];

function App() {
  const [form, setForm] = useState<FormState>(DEFAULT_FORM);
  const [result, setResult] = useState<SimulationResult>(() => simulateLocal(DEFAULT_FORM));
  const [loading, setLoading] = useState(false);
  const [activity, setActivity] = useState<ActivityStep[]>(INITIAL_ACTIVITY);
  const [alerts, setAlerts] = useState<string[]>([]);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [snapshots, setSnapshots] = useState<Snapshot[]>([]);
  const eventSourceRef = useRef<EventSource | null>(null);

  const charCount = form.text.length;

  const scoreColor = useMemo(() => {
    if (result.score >= 80) return "var(--accent)";
    if (result.score >= 60) return "#0f1419";
    if (result.score >= 40) return "#536471";
    return "#8b98a5";
  }, [result.score]);

  useEffect(() => {
    const stored = localStorage.getItem(SNAPSHOT_KEY);
    if (stored) {
      try {
        const parsed = JSON.parse(stored) as Snapshot[];
        setSnapshots(parsed);
      } catch {
        setSnapshots([]);
      }
    }
  }, []);

  useEffect(() => {
    localStorage.setItem(SNAPSHOT_KEY, JSON.stringify(snapshots.slice(0, 20)));
  }, [snapshots]);

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const snapshotParam = params.get("snapshot");
    if (!snapshotParam) return;
    const decoded = decodeSnapshot(snapshotParam);
    if (!decoded) return;
    setForm(decoded.input);
    setResult(decoded.output);
    setAlerts(["Loaded shared snapshot."]);
  }, []);

  const handleSimulate = async () => {
    setAlerts([]);
    if (!form.text.trim()) {
      setAlerts(["Add tweet text to simulate."]);
      return;
    }

    if (!form.useAi) {
      const local = simulateLocal(form);
      setResult(local);
      setActivity(INITIAL_ACTIVITY);
      appendLog("local", "Local simulation complete");
      return;
    }

    const requestId = generateRequestId();
    setLoading(true);
    setActivity([
      { label: "Preparing prompt", status: "active" },
      { label: "Calling Grok API", status: "pending" },
      { label: "Merging signals", status: "pending" },
    ]);
    appendLog("start", `Simulation ${requestId} started`);
    startStreaming(requestId);

    try {
      const response = await fetch("/api/simulate", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          request_id: requestId,
          text: form.text,
          media: form.media,
          has_link: form.hasLink,
          followers: form.followers,
          following: form.following,
          account_age_days: form.accountAgeDays,
          avg_engagement_rate: form.avgEngagementRate,
          posts_per_day: form.postsPerDay,
          verified: form.verified,
          hour_of_day: form.hourOfDay,
          novelty: form.novelty,
          timeliness: form.timeliness,
          topic_saturation: form.topicSaturation,
          audience_fit: form.audienceFit,
          controversy: form.controversy,
          sentiment: form.sentiment,
          use_ai: true,
        }),
      });

      if (!response.ok) {
        setAlerts([`AI server error: ${response.status}`]);
        setResult(simulateLocal(form));
        setActivity([
          { label: "Preparing prompt", status: "done" },
          { label: "Calling Grok API", status: "error" },
          { label: "Merging signals", status: "pending" },
        ]);
        setLoading(false);
        closeStream();
        return;
      }

      const data = await response.json();
      const normalized = normalizeApiResponse(data);
      setResult(normalized);
      setAlerts(normalized.warnings);
      appendLog("response", "Simulation results merged");
    } catch {
      setAlerts([
        "AI server not reachable. Run `cargo run -- serve` and open http://localhost:8787.",
      ]);
      setResult(simulateLocal(form));
      setActivity([
        { label: "Preparing prompt", status: "done" },
        { label: "Calling Grok API", status: "error" },
        { label: "Merging signals", status: "pending" },
      ]);
      setLoading(false);
      closeStream();
    }
  };

  const handleSaveSnapshot = () => {
    const snapshot: Snapshot = {
      id: generateSnapshotId(),
      createdAt: new Date().toISOString(),
      input: form,
      output: result,
    };
    setSnapshots((prev) => [snapshot, ...prev].slice(0, 20));
    appendLog("snapshot", `Snapshot saved (${snapshot.id})`);
  };

  const handleShareSnapshot = async () => {
    const snapshot: Snapshot = {
      id: generateSnapshotId(),
      createdAt: new Date().toISOString(),
      input: form,
      output: result,
    };
    const encoded = encodeSnapshot(snapshot);
    if (!encoded) {
      setAlerts(["Unable to generate share link."]);
      return;
    }
    const url = `${window.location.origin}${window.location.pathname}?snapshot=${encoded}`;
    try {
      await navigator.clipboard.writeText(url);
      setAlerts(["Share link copied to clipboard."]);
      appendLog("share", "Copied share link to clipboard");
    } catch {
      setAlerts(["Copy failed. You can manually copy the URL."]);
    }
  };

  const handleLoadSnapshot = (snapshot: Snapshot) => {
    setForm(snapshot.input);
    setResult(snapshot.output);
    setAlerts(["Snapshot loaded."]);
  };

  const handleDeleteSnapshot = (snapshotId: string) => {
    setSnapshots((prev) => prev.filter((item) => item.id !== snapshotId));
  };

  const clearLogs = () => setLogs([]);

  return (
    <div className="app">
      <header className="topbar">
        <div>
          <p className="eyebrow">X-style ranking sandbox</p>
          <h1>Virality Simulator</h1>
          <p className="subhead">
            Simulate how the For You ranking engine could react to a tweet draft.
          </p>
        </div>
        <div className="top-actions">
          <button className="ghost" onClick={handleSaveSnapshot}>
            Save snapshot
          </button>
          <button className="ghost" onClick={handleShareSnapshot}>
            Share snapshot
          </button>
        </div>
      </header>

      {alerts.length > 0 && (
        <div className="alert">
          {alerts.map((item) => (
            <span key={item}>{item}</span>
          ))}
        </div>
      )}

      <div className="main-grid">
        <section className="panel inputs">
          <div className="panel-header">
            <h2>Tweet inputs</h2>
            <label className="toggle">
              <input
                type="checkbox"
                checked={form.useAi}
                onChange={(event) =>
                  setForm({ ...form, useAi: event.target.checked })
                }
              />
              <span>Use Grok analysis</span>
            </label>
          </div>

          <div className="compose">
            <textarea
              value={form.text}
              onChange={(event) => setForm({ ...form, text: event.target.value })}
              placeholder="What's happening?"
            />
            <div className="compose-meta">
              <span>{charCount} chars</span>
              <span>AI mode requires local server</span>
            </div>
          </div>

          <div className="grid two">
            <Field label="Followers">
              <input
                type="number"
                value={form.followers}
                onChange={(event) =>
                  setForm({ ...form, followers: Number(event.target.value) })
                }
              />
            </Field>
            <Field label="Following">
              <input
                type="number"
                value={form.following}
                onChange={(event) =>
                  setForm({ ...form, following: Number(event.target.value) })
                }
              />
            </Field>
            <Field label="Account age (days)">
              <input
                type="number"
                value={form.accountAgeDays}
                onChange={(event) =>
                  setForm({ ...form, accountAgeDays: Number(event.target.value) })
                }
              />
            </Field>
            <Field label="Avg engagement rate">
              <input
                type="number"
                step="0.01"
                value={form.avgEngagementRate}
                onChange={(event) =>
                  setForm({
                    ...form,
                    avgEngagementRate: Number(event.target.value),
                  })
                }
              />
            </Field>
            <Field label="Posts per day">
              <input
                type="number"
                step="0.1"
                value={form.postsPerDay}
                onChange={(event) =>
                  setForm({ ...form, postsPerDay: Number(event.target.value) })
                }
              />
            </Field>
            <Field label="Post hour (0-23)">
              <input
                type="number"
                min={0}
                max={23}
                value={form.hourOfDay}
                onChange={(event) =>
                  setForm({ ...form, hourOfDay: Number(event.target.value) })
                }
              />
            </Field>
          </div>

          <div className="grid two">
            <Field label="Media type">
              <select
                value={form.media}
                onChange={(event) =>
                  setForm({ ...form, media: event.target.value as MediaType })
                }
              >
                <option value="none">None</option>
                <option value="image">Image</option>
                <option value="video">Video</option>
                <option value="gif">GIF</option>
              </select>
            </Field>
            <Field label="Preferences">
              <div className="inline-toggle">
                <label>
                  <input
                    type="checkbox"
                    checked={form.hasLink}
                    onChange={(event) =>
                      setForm({ ...form, hasLink: event.target.checked })
                    }
                  />
                  Includes link
                </label>
                <label>
                  <input
                    type="checkbox"
                    checked={form.verified}
                    onChange={(event) =>
                      setForm({ ...form, verified: event.target.checked })
                    }
                  />
                  Verified
                </label>
              </div>
            </Field>
          </div>

          <div className="slider-grid">
            <Slider
              label="Novelty"
              value={form.novelty}
              onChange={(value) => setForm({ ...form, novelty: value })}
            />
            <Slider
              label="Timeliness"
              value={form.timeliness}
              onChange={(value) => setForm({ ...form, timeliness: value })}
            />
            <Slider
              label="Audience fit"
              value={form.audienceFit}
              onChange={(value) => setForm({ ...form, audienceFit: value })}
            />
            <Slider
              label="Topic saturation"
              value={form.topicSaturation}
              onChange={(value) =>
                setForm({ ...form, topicSaturation: value })
              }
            />
            <Slider
              label="Controversy"
              value={form.controversy}
              onChange={(value) => setForm({ ...form, controversy: value })}
            />
            <Slider
              label="Sentiment"
              value={form.sentiment}
              min={-1}
              max={1}
              onChange={(value) => setForm({ ...form, sentiment: value })}
            />
          </div>

          <div className="compose-actions">
            <button className="primary" onClick={handleSimulate} disabled={loading}>
              {loading ? <span className="spinner" /> : "Simulate"}
            </button>
            <p className="hint">
              {loading
                ? "Grok is analyzing your tweet..."
                : "Run locally or use Grok for deeper analysis."}
            </p>
          </div>
        </section>

        <section className="panel score">
          <div className="score-header">
            <div>
              <p className="label">Virality score</p>
              <h2 style={{ color: scoreColor }}>{result.score.toFixed(1)}</h2>
              <span className="pill">{result.tier}</span>
            </div>
            <div className="score-meta">
              <span>Weighted score</span>
              <strong>{formatFloat(result.weightedScore, 2)}</strong>
              <span>Total impressions</span>
              <strong>{formatNumber(result.impressionsTotal)}</strong>
              <span>In-network</span>
              <strong>{formatNumber(result.impressionsIn)}</strong>
              <span>Out-of-network</span>
              <strong>{formatNumber(result.impressionsOon)}</strong>
            </div>
          </div>

          <div className="engagements">
            <div>
              <span>Unique engaged users</span>
              <strong>{formatNumber(result.expectedUniqueEngagements)}</strong>
              <small>{formatPercent(result.uniqueEngagementRate)}</small>
            </div>
            <div>
              <span>Total action volume</span>
              <strong>{formatNumber(result.expectedActionVolume)}</strong>
              <small>{formatFloat(result.actionVolumeRate, 2)} actions / impression</small>
            </div>
          </div>

          <div className="stats-grid">
            <Stat label="Likes" value={result.impressionsTotal * result.actions.like} />
            <Stat label="Replies" value={result.impressionsTotal * result.actions.reply} />
            <Stat label="Reposts" value={result.impressionsTotal * result.actions.repost} />
            <Stat label="Shares" value={result.impressionsTotal * result.actions.share} />
          </div>

          <div className="suggestions">
            <h3>Suggestions</h3>
            {result.suggestions.length === 0 ? (
              <p>No major blockers. Try A/B testing hooks.</p>
            ) : (
              <ul>
                {result.suggestions.map((item) => (
                  <li key={item}>{item}</li>
                ))}
              </ul>
            )}
          </div>
        </section>
      </div>

      <section className="panel under-hood">
        <div className="under-hood-header">
          <div>
            <h3>Under the hood</h3>
            <p>Live Grok status, transcript, and model signals.</p>
          </div>
          <button className="ghost" onClick={clearLogs}>
            Clear logs
          </button>
        </div>

        <div className="under-hood-grid">
          <div className="hood-card">
            <div className="card-header">
              <h4>Grok activity</h4>
              <span className={loading ? "live" : "idle"}>
                {loading ? "Running" : "Idle"}
              </span>
            </div>
            <ul className="activity">
              {activity.map((step) => (
                <li key={step.label} className={step.status}>
                  <span className="dot" />
                  {step.label}
                </li>
              ))}
            </ul>
            <div className="logs">
              {logs.length === 0 ? (
                <p>No live logs yet.</p>
              ) : (
                <ul>
                  {logs.map((entry) => (
                    <li key={entry.id}>
                      <span>{entry.timestamp}</span>
                      <strong>{entry.event}</strong>
                      <span>{entry.message}</span>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          </div>

          <div className="hood-card transcript">
            <div className="card-header">
              <h4>Grok transcript</h4>
              {result.llmTrace && (
                <span>{result.llmTrace.model}</span>
              )}
            </div>
            <div className="transcript-grid">
              <div>
                <h5>Parsed response</h5>
                {result.llm ? (
                  <div className="parsed-grid">
                    <div>
                      <span>Hook</span>
                      <strong>{formatFloat(result.llm.hook, 2)}</strong>
                    </div>
                    <div>
                      <span>Clarity</span>
                      <strong>{formatFloat(result.llm.clarity, 2)}</strong>
                    </div>
                    <div>
                      <span>Novelty</span>
                      <strong>{formatFloat(result.llm.novelty, 2)}</strong>
                    </div>
                    <div>
                      <span>Shareability</span>
                      <strong>{formatFloat(result.llm.shareability, 2)}</strong>
                    </div>
                    <div>
                      <span>Controversy</span>
                      <strong>{formatFloat(result.llm.controversy, 2)}</strong>
                    </div>
                    <div>
                      <span>Sentiment</span>
                      <strong>{formatFloat(result.llm.sentiment, 2)}</strong>
                    </div>
                    <div className="parsed-list">
                      <span>Suggestions</span>
                      <ul>
                        {result.llm.suggestions.map((item) => (
                          <li key={item}>{item}</li>
                        ))}
                      </ul>
                    </div>
                  </div>
                ) : (
                  <p className="muted">Run Grok to see parsed signals.</p>
                )}
              </div>
              <div>
                <h5>Raw response</h5>
                {result.llmTrace ? (
                  <pre>{result.llmTrace.raw_response}</pre>
                ) : (
                  <p className="muted">No raw response yet.</p>
                )}
              </div>
            </div>
            {result.llmTrace && (
              <div className="trace-meta">
                <span>Latency: {result.llmTrace.latency_ms} ms</span>
                <span>Prompt: {result.llmTrace.prompt_summary}</span>
                <span>Tokens: {formatOptional(result.llmTrace.total_tokens)}</span>
              </div>
            )}
          </div>

          <div className="hood-card">
            <div className="card-header">
              <h4>Signals</h4>
            </div>
            <SignalBar label="Quality" value={result.signals.contentQuality} />
            <SignalBar label="Hook" value={result.signals.hook} />
            <SignalBar label="Author" value={result.signals.authorQuality} />
            <SignalBar label="Audience" value={result.signals.audienceAlignment} />
            <SignalBar label="Share" value={result.signals.shareability} />
            <SignalBar label="Risk" value={result.signals.negativeRisk} />
          </div>

          <div className="hood-card">
            <div className="card-header">
              <h4>Action probabilities</h4>
            </div>
            <div className="prob-grid">
              {Object.entries(result.actions).map(([key, value]) => (
                <div key={key}>
                  <span>{formatLabel(key)}</span>
                  <strong>{formatPercent(value)}</strong>
                </div>
              ))}
            </div>
          </div>

          <div className="hood-card">
            <div className="card-header">
              <h4>Snapshots</h4>
            </div>
            {snapshots.length === 0 ? (
              <p className="muted">No snapshots saved yet.</p>
            ) : (
              <ul className="snapshot-list">
                {snapshots.map((snapshot) => (
                  <li key={snapshot.id}>
                    <div>
                      <strong>{snapshot.id}</strong>
                      <span>{formatDate(snapshot.createdAt)}</span>
                    </div>
                    <div className="snapshot-actions">
                      <button onClick={() => handleLoadSnapshot(snapshot)}>Load</button>
                      <button onClick={() => handleDeleteSnapshot(snapshot.id)}>Delete</button>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>
      </section>
    </div>
  );

  function startStreaming(requestId: string) {
    closeStream();
    const source = new EventSource(`/api/simulate/stream?request_id=${requestId}`);
    eventSourceRef.current = source;

    source.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data) as {
          event: string;
          message: string;
          timestamp_ms: number;
        };
        appendLog(data.event, data.message, data.timestamp_ms);
        handleStreamEvent(data.event);
      } catch {
        appendLog("stream", "Received malformed stream event");
      }
    };

    source.onerror = () => {
      appendLog("error", "Stream connection lost");
      setLoading(false);
      closeStream();
    };
  }

  function closeStream() {
    if (eventSourceRef.current) {
      eventSourceRef.current.close();
      eventSourceRef.current = null;
    }
  }

  function handleStreamEvent(event: string) {
    if (event === "connected") {
      return;
    }
    if (event === "start") {
      updateActivity(0, "active");
      return;
    }
    if (event === "calling") {
      setActivity([
        { label: "Preparing prompt", status: "done" },
        { label: "Calling Grok API", status: "active" },
        { label: "Merging signals", status: "pending" },
      ]);
      return;
    }
    if (event === "received") {
      setActivity([
        { label: "Preparing prompt", status: "done" },
        { label: "Calling Grok API", status: "done" },
        { label: "Merging signals", status: "active" },
      ]);
      return;
    }
    if (event === "merge") {
      setActivity([
        { label: "Preparing prompt", status: "done" },
        { label: "Calling Grok API", status: "done" },
        { label: "Merging signals", status: "active" },
      ]);
      return;
    }
    if (event === "done") {
      setActivity([
        { label: "Preparing prompt", status: "done" },
        { label: "Calling Grok API", status: "done" },
        { label: "Merging signals", status: "done" },
      ]);
      setLoading(false);
      closeStream();
      return;
    }
    if (event === "error") {
      setActivity([
        { label: "Preparing prompt", status: "done" },
        { label: "Calling Grok API", status: "error" },
        { label: "Merging signals", status: "pending" },
      ]);
      setLoading(false);
      closeStream();
    }
  }

  function appendLog(event: string, message: string, timestampMs?: number) {
    const time = timestampMs ? new Date(timestampMs).toLocaleTimeString() : new Date().toLocaleTimeString();
    const entry: LogEntry = {
      id: generateSnapshotId(),
      timestamp: time,
      message,
      event,
    };
    setLogs((prev) => [entry, ...prev].slice(0, LOG_LIMIT));
  }

  function updateActivity(index: number, status: ActivityStep["status"]) {
    setActivity((prev) =>
      prev.map((step, i) => (i === index ? { ...step, status } : step))
    );
  }
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="field">
      <span>{label}</span>
      {children}
    </label>
  );
}

function Slider({
  label,
  value,
  min = 0,
  max = 1,
  onChange,
}: {
  label: string;
  value: number;
  min?: number;
  max?: number;
  onChange: (value: number) => void;
}) {
  return (
    <label className="slider">
      <div>
        <span>{label}</span>
        <strong>{value.toFixed(2)}</strong>
      </div>
      <input
        type="range"
        min={min}
        max={max}
        step={0.01}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
      />
    </label>
  );
}

function Stat({ label, value }: { label: string; value: number }) {
  return (
    <div className="stat">
      <span>{label}</span>
      <strong>{formatNumber(value)}</strong>
    </div>
  );
}

function SignalBar({ label, value }: { label: string; value: number }) {
  return (
    <div className="signal">
      <span>{label}</span>
      <div className="bar">
        <span style={{ width: `${Math.round(value * 100)}%` }} />
      </div>
      <strong>{value.toFixed(2)}</strong>
    </div>
  );
}

function normalizeApiResponse(raw: any): SimulationResult {
  const actionsRaw = raw.actions || {};
  const signalsRaw = raw.signals || {};

  return {
    score: raw.score ?? 0,
    tier: raw.tier ?? "Low",
    weightedScore: raw.weighted_score ?? 0,
    impressionsIn: raw.impressions_in ?? 0,
    impressionsOon: raw.impressions_oon ?? 0,
    impressionsTotal: raw.impressions_total ?? 0,
    expectedUniqueEngagements: raw.expected_unique_engagements ?? 0,
    expectedActionVolume: raw.expected_action_volume ?? 0,
    uniqueEngagementRate: raw.unique_engagement_rate ?? 0,
    actionVolumeRate: raw.action_volume_rate ?? 0,
    actions: {
      like: actionsRaw.like ?? 0,
      reply: actionsRaw.reply ?? 0,
      repost: actionsRaw.repost ?? 0,
      quote: actionsRaw.quote ?? 0,
      click: actionsRaw.click ?? 0,
      profileClick: actionsRaw.profile_click ?? 0,
      videoView: actionsRaw.video_view ?? 0,
      photoExpand: actionsRaw.photo_expand ?? 0,
      share: actionsRaw.share ?? 0,
      dwell: actionsRaw.dwell ?? 0,
      followAuthor: actionsRaw.follow_author ?? 0,
      notInterested: actionsRaw.not_interested ?? 0,
      block: actionsRaw.block ?? 0,
      mute: actionsRaw.mute ?? 0,
      report: actionsRaw.report ?? 0,
    },
    signals: {
      contentQuality: signalsRaw.content_quality ?? 0,
      hook: signalsRaw.hook ?? 0,
      authorQuality: signalsRaw.author_quality ?? 0,
      audienceAlignment: signalsRaw.audience_alignment ?? 0,
      negativeRisk: signalsRaw.negative_risk ?? 0,
      shareability: signalsRaw.shareability ?? 0,
      clarity: signalsRaw.clarity ?? 0,
      lengthScore: signalsRaw.length_score ?? 0,
      mediaScore: signalsRaw.media_score ?? 0,
      timeScore: signalsRaw.time_score ?? 0,
      novelty: signalsRaw.novelty ?? 0,
      timeliness: signalsRaw.timeliness ?? 0,
    },
    suggestions: raw.suggestions ?? [],
    warnings: raw.warnings ?? [],
    llm: raw.llm
      ? {
          hook: raw.llm.hook,
          clarity: raw.llm.clarity,
          novelty: raw.llm.novelty,
          shareability: raw.llm.shareability,
          controversy: raw.llm.controversy,
          sentiment: raw.llm.sentiment,
          suggestions: raw.llm.suggestions ?? [],
        }
      : undefined,
    llmTrace: raw.llm_trace
      ? {
          model: raw.llm_trace.model,
          latency_ms: raw.llm_trace.latency_ms,
          prompt_summary: raw.llm_trace.prompt_summary,
          raw_response: raw.llm_trace.raw_response,
          prompt_tokens: raw.llm_trace.prompt_tokens,
          completion_tokens: raw.llm_trace.completion_tokens,
          total_tokens: raw.llm_trace.total_tokens,
        }
      : undefined,
    requestId: raw.request_id,
  };
}

function simulateLocal(form: FormState): SimulationResult {
  const features = extractTextFeatures(form.text);
  const mediaScore = mediaScoreFor(form.media);
  const hasLink = form.hasLink || features.urls > 0;
  const linkFlag = hasLink ? 1 : 0;

  const lengthScore = gaussian(features.charCount, 140, 70);
  const readabilityScore = gaussian(features.avgWordLen, 5, 2);

  const exclaimFactor = clamp01(features.exclamations / 3);
  const hashtagFactor = clamp01(features.hashtags / 5);
  const mentionFactor = clamp01(features.mentions / 4);

  const spamminess = clamp01(
    0.2 * exclaimFactor +
      0.25 * hashtagFactor +
      0.2 * mentionFactor +
      0.3 * clamp01(features.uppercaseRatio / 0.4) +
      0.2 * linkFlag
  );

  const hook = clamp01(
    0.35 * boolToNumber(features.questions > 0) +
      0.2 * boolToNumber(features.exclamations > 0) +
      0.25 * boolToNumber(features.startsWithNumber) +
      0.2 * boolToNumber(features.hasHookWord)
  );

  const clarity = clamp01(0.5 * lengthScore + 0.3 * readabilityScore + 0.2 * (1 - spamminess));
  const novelty = clamp01(form.novelty);
  const timeliness = clamp01(form.timeliness);

  const shareability = clamp01(
    0.4 * hook +
      0.3 * novelty +
      0.2 * clarity +
      0.1 * boolToNumber(features.ctaShare)
  );

  const contentQuality = clamp01(0.45 * clarity + 0.25 * hook + 0.2 * novelty + 0.1 * timeliness);

  const followersLog = log10Safe(form.followers + 1);
  const authorStrength = clamp01((followersLog - 2) / 3);

  const ratio = form.following === 0 ? 1 : form.followers / form.following;
  const ratioScore = clamp01(log10Safe(ratio + 1) / 2);

  const ageYears = form.accountAgeDays / 365;
  const ageScore = clamp01(ageYears / 5);

  const engScore = clamp01(form.avgEngagementRate / 0.06);
  const cadenceScore = gaussian(form.postsPerDay, 2, 2.5);
  const verifiedBonus = form.verified ? 0.1 : 0;

  const authorQuality = clamp01(
    0.35 * authorStrength +
      0.2 * ageScore +
      0.2 * engScore +
      0.15 * ratioScore +
      0.1 * cadenceScore +
      verifiedBonus
  );

  const topicSaturation = clamp01(form.topicSaturation);
  const audienceFit = clamp01(form.audienceFit);
  const audienceAlignment = clamp01(
    0.6 * audienceFit + 0.2 * (1 - topicSaturation) + 0.2 * ratioScore
  );

  const negativeSentiment = Math.max(0, -form.sentiment);
  const capsRisk = clamp01(features.uppercaseRatio / 0.35) * 0.2;
  const negativeRisk = clamp01(
    0.4 * clamp01(form.controversy) +
      0.25 * spamminess +
      0.15 * negativeSentiment +
      capsRisk +
      0.1 * topicSaturation
  );

  const positiveSignal = clamp01(0.4 * contentQuality + 0.35 * authorQuality + 0.25 * audienceAlignment);
  const viralLift = clamp01(0.5 * hook + 0.3 * novelty + 0.2 * mediaScore);

  const base = -2.0 + 3.2 * positiveSignal + 1.4 * viralLift - 2.2 * negativeRisk;

  const hasQuestion = boolToNumber(features.questions > 0);
  const ctaReply = boolToNumber(features.ctaReply);
  const ctaShare = boolToNumber(features.ctaShare);
  const isVideo = form.media === "video" ? 1 : 0;
  const isImage = form.media === "image" || form.media === "gif" ? 1 : 0;

  const actions: ActionProbs = {
    like: sigmoid(base + 0.6 * mediaScore + 0.2 * Math.max(0, form.sentiment)),
    reply: sigmoid(base - 0.2 * mediaScore + 0.6 * hasQuestion + 0.3 * form.controversy + 0.2 * ctaReply),
    repost: sigmoid(base + 0.6 * shareability + 0.3 * novelty - 0.3 * linkFlag + 0.1 * ctaShare),
    quote: sigmoid(base + 0.4 * form.controversy + 0.2 * novelty),
    click: sigmoid(base + 0.9 * linkFlag + 0.2 * hook),
    profileClick: sigmoid(base + 0.5 * authorQuality + 0.2 * novelty),
    videoView: sigmoid(base + 1.2 * isVideo + 0.2 * hook),
    photoExpand: sigmoid(base + 1.0 * isImage + 0.1 * hook),
    share: sigmoid(base + 0.5 * shareability + 0.2 * novelty),
    dwell: sigmoid(base + 0.2 * lengthScore + 0.4 * mediaScore - 0.2 * linkFlag),
    followAuthor: sigmoid(base + 0.6 * authorQuality + 0.2 * hook),
    notInterested: sigmoid(-1.0 + 2.2 * negativeRisk + 0.6 * topicSaturation - 0.8 * audienceAlignment),
    block: sigmoid(-2.0 + 2.6 * negativeRisk + 0.6 * form.controversy),
    mute: sigmoid(-1.8 + 2.3 * negativeRisk + 0.8 * topicSaturation),
    report: sigmoid(-2.4 + 2.8 * negativeRisk + 0.6 * form.controversy),
  };

  const weightedScore = weightedScoreFrom(actions);

  const timeScore = timeOfDayScore(form.hourOfDay);
  const activeFraction = 0.015 + 0.08 * timeScore;
  const impressionsIn = form.followers * activeFraction * Math.max(0, 0.6 + 0.4 * audienceAlignment);

  const oonSeed = 300 + 1400 * positiveSignal;
  const oonMultiplier = 1 + clamp01((weightedScore - 1) / 3) * 4;
  let impressionsOon =
    oonSeed * oonMultiplier * (0.5 + 0.5 * contentQuality) * (1 - 0.7 * topicSaturation) * (1 - 0.5 * negativeRisk);
  if (!Number.isFinite(impressionsOon) || impressionsOon < 0) impressionsOon = 0;

  const impressionsTotal = impressionsIn + impressionsOon;
  const actionVolumeRateValue = actionVolumeRate(actions);
  const uniqueEngagementRateValue = uniqueEngagementRate(actions);
  const expectedActionVolume = impressionsTotal * actionVolumeRateValue;
  const expectedUniqueEngagements = impressionsTotal * uniqueEngagementRateValue;

  const raw = (weightedScore - 1) * 0.8 + (log10Safe(impressionsTotal + 1) - 3) * 0.4;
  const score = 100 * sigmoid(raw);
  const tier = tierFromScore(score);

  const signals: Signals = {
    contentQuality,
    hook,
    authorQuality,
    audienceAlignment,
    negativeRisk,
    shareability,
    clarity,
    lengthScore,
    mediaScore,
    timeScore,
    novelty,
    timeliness,
  };

  const suggestions = buildSuggestions(form, features, signals, actions, weightedScore);

  return {
    score,
    tier,
    weightedScore,
    impressionsIn,
    impressionsOon,
    impressionsTotal,
    expectedUniqueEngagements,
    expectedActionVolume,
    uniqueEngagementRate: uniqueEngagementRateValue,
    actionVolumeRate: actionVolumeRateValue,
    actions,
    signals,
    suggestions,
    warnings: [],
  };
}

function extractTextFeatures(text: string) {
  let hashtags = 0;
  let mentions = 0;
  let questions = 0;
  let exclamations = 0;
  let emojiCount = 0;
  let uppercase = 0;
  let letters = 0;

  for (const ch of text) {
    if (ch === "#") hashtags += 1;
    if (ch === "@") mentions += 1;
    if (ch === "?") questions += 1;
    if (ch === "!") exclamations += 1;
    if (ch.codePointAt(0) && ch.codePointAt(0)! > 127) emojiCount += 1;

    if (isAsciiLetter(ch)) {
      letters += 1;
      if (isUpper(ch)) uppercase += 1;
    }
  }

  const lowercase = text.toLowerCase();
  const urls = countMatches(lowercase, "http://") + countMatches(lowercase, "https://") + countMatches(lowercase, "www.");

  let wordTotal = 0;
  let wordCount = 0;
  for (const word of text.split(/\s+/)) {
    const len = Array.from(word).filter((c) => isAsciiLetter(c)).length;
    if (len > 0) {
      wordTotal += len;
      wordCount += 1;
    }
  }

  const avgWordLen = wordCount === 0 ? 0 : wordTotal / wordCount;
  const uppercaseRatio = letters === 0 ? 0 : uppercase / letters;
  const startsWithNumber = /^\s*\d/.test(text);

  const hookWords = [
    "how",
    "why",
    "what",
    "stop",
    "new",
    "breaking",
    "secret",
    "tips",
    "guide",
    "learn",
    "thread",
    "facts",
    "proof",
    "mistakes",
    "warning",
  ];
  const hasHookWord = hookWords.some((word) => lowercase.includes(word));

  const ctaShare = ["retweet", "repost", "share", "rt ", "boost"].some((word) => lowercase.includes(word));
  const ctaReply = ["thoughts", "what do you think", "agree", "disagree", "reply", "comment"].some(
    (word) => lowercase.includes(word)
  );

  return {
    charCount: Array.from(text).length,
    wordCount,
    hashtags,
    mentions,
    urls,
    questions,
    exclamations,
    emojiCount,
    uppercaseRatio,
    avgWordLen,
    startsWithNumber,
    hasHookWord,
    ctaShare,
    ctaReply,
  };
}

function buildSuggestions(
  form: FormState,
  features: ReturnType<typeof extractTextFeatures>,
  signals: Signals,
  actions: ActionProbs,
  weightedScore: number
) {
  const suggestions: string[] = [];

  if (features.charCount < 50) {
    suggestions.push("Add a clearer hook and more context; aim for ~80-200 characters.");
  }
  if (features.charCount > 260) {
    suggestions.push("Trim to under ~220 characters to improve early engagement velocity.");
  }
  if (features.hashtags > 3) {
    suggestions.push("Reduce hashtags to 1-2; too many can look spammy.");
  }
  if (features.mentions > 2) {
    suggestions.push("Limit mentions; too many can reduce reach and clarity.");
  }
  if (form.hasLink || features.urls > 0) {
    suggestions.push("Links often reduce in-feed engagement; consider moving the link to a reply.");
  }
  if (form.media === "none" && features.charCount < 160) {
    suggestions.push("Consider adding an image or video to boost dwell and shares.");
  }
  if (signals.hook < 0.35) {
    suggestions.push("Strengthen the first line with a question, surprising stat, or bold claim.");
  }
  if (signals.shareability < 0.4) {
    suggestions.push("Make it more shareable: concise takeaway, list, or strong opinion.");
  }
  if (signals.clarity < 0.5) {
    suggestions.push("Simplify wording; shorter words and fewer clauses improve scanability.");
  }
  if (features.uppercaseRatio > 0.3) {
    suggestions.push("Reduce ALL CAPS; it increases negative feedback signals.");
  }
  if (signals.negativeRisk > 0.55) {
    suggestions.push("Tone down contentious framing to reduce not-interested/report signals.");
  }
  if (form.topicSaturation > 0.6) {
    suggestions.push("High topic saturation; use a unique angle or niche framing.");
  }
  if (form.audienceFit < 0.5) {
    suggestions.push("Align the topic with follower interests to boost initial velocity.");
  }
  if (signals.timeScore < 0.4) {
    suggestions.push("Post during peak hours (around 9-11am or 7-9pm local).");
  }
  if (weightedScore < 0.8) {
    suggestions.push("Focus on increasing repost/share intent; that is a main driver of out-of-network reach.");
  }
  if (actions.reply < 0.02 && features.questions === 0) {
    suggestions.push("Invite replies with a direct question or a clear prompt.");
  }

  return suggestions.slice(0, 10);
}

function actionVolumeRate(actions: ActionProbs) {
  return positiveActionProbs(actions).reduce((sum, value) => sum + value, 0);
}

function uniqueEngagementRate(actions: ActionProbs) {
  let noneProbability = 1;
  positiveActionProbs(actions).forEach((value) => {
    noneProbability *= 1 - clamp01(value);
  });
  return clamp01(1 - noneProbability);
}

function positiveActionProbs(actions: ActionProbs) {
  return [
    actions.like,
    actions.reply,
    actions.repost,
    actions.quote,
    actions.share,
    actions.click,
    actions.profileClick,
    actions.followAuthor,
    actions.videoView,
    actions.photoExpand,
  ];
}

function weightedScoreFrom(actions: ActionProbs) {
  return (
    actions.like * 1.0 +
    actions.reply * 1.6 +
    actions.repost * 2.0 +
    actions.quote * 1.7 +
    actions.click * 0.4 +
    actions.profileClick * 0.3 +
    actions.videoView * 0.5 +
    actions.photoExpand * 0.3 +
    actions.share * 1.4 +
    actions.dwell * 0.2 +
    actions.followAuthor * 1.2 +
    actions.notInterested * -2.5 +
    actions.block * -5.0 +
    actions.mute * -3.0 +
    actions.report * -6.0
  );
}

function tierFromScore(score: number) {
  if (score < 35) return "Low";
  if (score < 55) return "Moderate";
  if (score < 75) return "High";
  if (score < 90) return "Very High";
  return "Breakout";
}

function mediaScoreFor(media: MediaType) {
  switch (media) {
    case "image":
      return 0.4;
    case "gif":
      return 0.6;
    case "video":
      return 0.8;
    default:
      return 0.0;
  }
}

function timeOfDayScore(hour: number) {
  const morning = gaussian(hour, 9, 3.5);
  const evening = gaussian(hour, 20, 3.5);
  return Math.max(morning, evening);
}

function gaussian(x: number, center: number, width: number) {
  if (width <= 0) return 0;
  const z = (x - center) / width;
  return Math.exp(-z * z);
}

function sigmoid(x: number) {
  return 1 / (1 + Math.exp(-x));
}

function clamp01(value: number) {
  if (!Number.isFinite(value)) return 0;
  return Math.min(1, Math.max(0, value));
}

function log10Safe(value: number) {
  if (value <= 0) return 0;
  return Math.log10(value);
}

function boolToNumber(value: boolean) {
  return value ? 1 : 0;
}

function formatNumber(value: number) {
  const rounded = Math.max(0, Math.round(value));
  return rounded.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",");
}

function formatPercent(value: number) {
  return `${(value * 100).toFixed(1)}%`;
}

function formatFloat(value: number, digits: number) {
  return value.toFixed(digits);
}

function formatLabel(value: string) {
  return value.replace(/([A-Z])/g, " $1").replace(/^\w/, (c) => c.toUpperCase());
}

function formatOptional(value?: number | null) {
  if (value === undefined || value === null) return "-";
  return String(value);
}

function formatDate(value: string) {
  return new Date(value).toLocaleString();
}

function countMatches(text: string, needle: string) {
  if (!needle) return 0;
  return text.split(needle).length - 1;
}

function isAsciiLetter(ch: string) {
  return /[A-Za-z]/.test(ch);
}

function isUpper(ch: string) {
  return /[A-Z]/.test(ch);
}

function generateSnapshotId() {
  return Math.random().toString(36).slice(2, 8);
}

function generateRequestId() {
  return `req-${Date.now()}-${Math.random().toString(16).slice(2, 6)}`;
}

function encodeSnapshot(snapshot: Snapshot) {
  try {
    const json = JSON.stringify(snapshot);
    return btoa(unescape(encodeURIComponent(json)));
  } catch {
    return null;
  }
}

function decodeSnapshot(encoded: string) {
  try {
    const json = decodeURIComponent(escape(atob(encoded)));
    return JSON.parse(json) as Snapshot;
  } catch {
    return null;
  }
}

export default App;
