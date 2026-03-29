import { useEffect, useMemo, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import {
  AppError,
  RiverSolveRequest,
  RiverSolveResponse,
  ValidateConfigResponse,
  emptyRiverSolveRequest,
  loadSampleRequest,
  solveRiverSpot,
  validateConfig,
  writeE2eSmokeReport,
} from './api';

const integerFields = [
  ['potSize', 'Pot size'],
  ['oopStack', 'OOP stack'],
  ['ipStack', 'IP stack'],
  ['smallBlind', 'Small blind'],
  ['bigBlind', 'Big blind'],
  ['firstBetSize', 'First bet size'],
  ['afterCheckBetSize', 'After-check bet size'],
  ['iterations', 'Iterations'],
  ['deterministicSeed', 'Deterministic seed'],
] as const;

type BusyState = 'validate' | 'solve' | 'load-sample' | null;

function formatDecimal(value: number): string {
  return value.toFixed(6);
}

function formatPercent(value: number): string {
  return `${(value * 100).toFixed(2)}%`;
}

function toErrorMessage(error: AppError | Error): string {
  if ('code' in error) {
    return `${error.code}: ${error.message}`;
  }
  return error.message;
}

export default function App() {
  const smokeEnabled = import.meta.env.VITE_POKIE_E2E === '1';
  const [request, setRequest] = useState<RiverSolveRequest>(emptyRiverSolveRequest());
  const [validation, setValidation] = useState<ValidateConfigResponse | null>(null);
  const [result, setResult] = useState<RiverSolveResponse | null>(null);
  const [busy, setBusy] = useState<BusyState>('load-sample');
  const [status, setStatus] = useState('Loading the exact river sample spot…');
  const [error, setError] = useState<string | null>(null);
  const [smokeStarted, setSmokeStarted] = useState(false);
  const [sampleReady, setSampleReady] = useState(false);

  useEffect(() => {
    if (smokeEnabled) {
      void writeE2eSmokeReport({ status: 'running', stage: 'boot' });
    }
  }, [smokeEnabled]);

  useEffect(() => {
    loadSampleRequest()
      .then((sample) => {
        setRequest(sample);
        setStatus('Sample river-only spot loaded from the desktop backend.');
        setSampleReady(true);
        if (smokeEnabled) {
          void writeE2eSmokeReport({ status: 'running', stage: 'sample-loaded' });
        }
      })
      .catch((nextError) => {
        setRequest(emptyRiverSolveRequest());
        setSampleReady(false);
        setStatus('Could not load the backend sample spot. The form is ready for manual input.');
        setError(toErrorMessage(nextError as AppError));
        if (smokeEnabled) {
          void setSmokeWindowTitle('Pokie E2E FAIL');
          void writeE2eSmokeReport({
            status: 'fail',
            stage: 'load-sample',
            error: toErrorMessage(nextError as AppError),
          });
        }
      })
      .finally(() => setBusy(null));
  }, [smokeEnabled]);

  const canSubmit = busy === null;
  const summaryItems = useMemo(
    () => [
      ['Slice', 'HU NLHE cash · river-only · single-bet/no-raise'],
      ['Surface', 'validate_config + solve_river_spot'],
      ['Board', request.board || '—'],
      ['Actor', request.playerToAct.toUpperCase()],
    ],
    [request.board, request.playerToAct],
  );

  function updateField<Key extends keyof RiverSolveRequest>(key: Key, value: RiverSolveRequest[Key]) {
    setRequest((current) => ({ ...current, [key]: value }));
  }

  async function setSmokeWindowTitle(title: string) {
    if (!smokeEnabled) {
      return;
    }
    document.title = title;
    try {
      await getCurrentWindow().setTitle(title);
    } catch {
      // ignore title updates outside the Tauri shell
    }
  }

  function applySolveResponse(solveResponse: RiverSolveResponse) {
    setValidation({
      configHash: solveResponse.configHash,
      compatibleDealCount: solveResponse.compatibleDealCount,
      normalized: solveResponse.normalized,
    });
    setResult(solveResponse);
  }

  async function handleLoadSample() {
    setBusy('load-sample');
    setError(null);
    try {
      const sample = await loadSampleRequest();
      setRequest(sample);
      setValidation(null);
      setResult(null);
      setStatus('Sample river-only spot loaded from the desktop backend.');
    } catch (nextError) {
      setValidation(null);
      setResult(null);
      setStatus('Could not reload the backend sample spot.');
      setError(toErrorMessage(nextError as AppError));
    } finally {
      setBusy(null);
    }
  }

  async function handleValidate() {
    setBusy('validate');
    setError(null);
    setResult(null);
    try {
      const response = await validateConfig(request);
      setValidation(response);
      setStatus('Config validated and canonicalized by the Rust backend.');
    } catch (nextError) {
      setValidation(null);
      setStatus('Validation failed.');
      setError(toErrorMessage(nextError as AppError));
    } finally {
      setBusy(null);
    }
  }

  async function handleSolve() {
    setBusy('solve');
    setError(null);
    try {
      const solveResponse = await solveRiverSpot(request);
      applySolveResponse(solveResponse);
      setStatus('Solve completed through the exact river backend slice.');
    } catch (nextError) {
      setResult(null);
      setStatus('Solve failed.');
      setError(toErrorMessage(nextError as AppError));
    } finally {
      setBusy(null);
    }
  }

  useEffect(() => {
    if (!smokeEnabled || smokeStarted || !sampleReady) {
      return;
    }

    setSmokeStarted(true);
    void setSmokeWindowTitle('Pokie E2E RUNNING');

    async function runSmokeFlow() {
      setBusy('validate');
      setError(null);
      setResult(null);
      setStatus('Running desktop E2E smoke flow…');
      await writeE2eSmokeReport({ status: 'running', stage: 'validate' });

      try {
        const validationResponse = await validateConfig(request);
        setValidation(validationResponse);

        setBusy('solve');
        await writeE2eSmokeReport({ status: 'running', stage: 'solve' });
        const solveResponse = await solveRiverSpot(request);
        applySolveResponse(solveResponse);
        setStatus('Desktop E2E smoke flow passed.');
        await setSmokeWindowTitle('Pokie E2E PASS');
        await writeE2eSmokeReport({
          status: 'pass',
          configHash: solveResponse.configHash,
          treeIdentity: solveResponse.treeIdentity,
          iterations: solveResponse.iterations,
          nashConv: solveResponse.nashConv,
        });
      } catch (nextError) {
        setValidation(null);
        setResult(null);
        setStatus('Desktop E2E smoke flow failed.');
        setError(toErrorMessage(nextError as AppError));
        await setSmokeWindowTitle('Pokie E2E FAIL');
        await writeE2eSmokeReport({
          status: 'fail',
          stage: 'validate-or-solve',
          error: toErrorMessage(nextError as AppError),
        });
      } finally {
        setBusy(null);
      }
    }

    void runSmokeFlow();
  }, [request, sampleReady, smokeEnabled, smokeStarted]);

  return (
    <main className="app-shell">
      <section className="hero card">
        <div>
          <p className="eyebrow">Pokie desktop slice</p>
          <h1>Exact river conformance-backed desktop shell</h1>
          <p className="lede">
            This app is intentionally narrow: it validates and solves the current exact heads-up NLHE
            river slice through the app-api boundary.
          </p>
        </div>
        <div className="summary-grid">
          {summaryItems.map(([label, value]) => (
            <div className="summary-item" key={label}>
              <span>{label}</span>
              <strong>{value}</strong>
            </div>
          ))}
        </div>
      </section>

      <section className="content-grid">
        <section className="card">
          <div className="section-header">
            <div>
              <p className="eyebrow">Scenario builder</p>
              <h2>River solve request</h2>
            </div>
            <div className="button-row">
              <button type="button" onClick={handleLoadSample} disabled={!canSubmit}>
                Load sample
              </button>
              <button type="button" onClick={handleValidate} disabled={!canSubmit}>
                Validate
              </button>
              <button type="button" className="primary" onClick={handleSolve} disabled={!canSubmit}>
                Solve
              </button>
            </div>
          </div>

          <label className="field">
            <span>Board</span>
            <input value={request.board} onChange={(event) => updateField('board', event.target.value)} />
          </label>

          <div className="two-column">
            <label className="field">
              <span>OOP range</span>
              <textarea
                rows={4}
                value={request.oopRange}
                onChange={(event) => updateField('oopRange', event.target.value)}
              />
            </label>
            <label className="field">
              <span>IP range</span>
              <textarea
                rows={4}
                value={request.ipRange}
                onChange={(event) => updateField('ipRange', event.target.value)}
              />
            </label>
          </div>

          <div className="field-grid">
            {integerFields.map(([key, label]) => (
              <label className="field" key={key}>
                <span>{label}</span>
                <input
                  type="text"
                  inputMode="numeric"
                  pattern="[0-9]*"
                  value={request[key]}
                  onChange={(event) => updateField(key, event.target.value)}
                />
              </label>
            ))}
            <label className="field">
              <span>Player to act</span>
              <select
                value={request.playerToAct}
                onChange={(event) => updateField('playerToAct', event.target.value as RiverSolveRequest['playerToAct'])}
              >
                <option value="oop">OOP</option>
                <option value="ip">IP</option>
              </select>
            </label>
          </div>
        </section>

        <section className="stack">
          <section className="card status-card">
            <p className="eyebrow">Run status</p>
            <h2>{busy ? 'Working…' : 'Ready'}</h2>
            <p>{status}</p>
            {error ? <pre className="error-block">{error}</pre> : null}
          </section>

          <section className="card">
            <p className="eyebrow">Validation</p>
            <h2>Canonical config</h2>
            {validation ? (
              <>
                <dl className="metric-list">
                  <div>
                    <dt>Config hash</dt>
                    <dd>{validation.configHash}</dd>
                  </div>
                  <div>
                    <dt>Compatible deals</dt>
                    <dd>{validation.compatibleDealCount}</dd>
                  </div>
                </dl>
                <pre className="json-block">{JSON.stringify(validation.normalized, null, 2)}</pre>
              </>
            ) : (
              <p className="placeholder">Validate a request to inspect the canonicalized backend config.</p>
            )}
          </section>

          <section className="card">
            <p className="eyebrow">Solve result</p>
            <h2>Root EV and exploitability</h2>
            {result ? (
              <>
                <dl className="metric-list">
                  <div>
                    <dt>Config hash</dt>
                    <dd>{result.configHash}</dd>
                  </div>
                  <div>
                    <dt>Tree identity</dt>
                    <dd>{result.treeIdentity}</dd>
                  </div>
                  <div>
                    <dt>Iterations</dt>
                    <dd>{result.iterations}</dd>
                  </div>
                  <div>
                    <dt>Root EV (OOP)</dt>
                    <dd>{formatDecimal(result.rootValueOop)}</dd>
                  </div>
                  <div>
                    <dt>Nash conv</dt>
                    <dd>{formatDecimal(result.nashConv)}</dd>
                  </div>
                  <div>
                    <dt>P0 improvement</dt>
                    <dd>{formatDecimal(result.p0Improvement)}</dd>
                  </div>
                  <div>
                    <dt>P1 improvement</dt>
                    <dd>{formatDecimal(result.p1Improvement)}</dd>
                  </div>
                </dl>

                <div className="infoset-grid">
                  {result.rootInfosets.map((infoset) => (
                    <article className="infoset-card" key={infoset.key}>
                      <header>
                        <strong>{infoset.key}</strong>
                      </header>
                      <ul>
                        {infoset.actions.map((action) => (
                          <li key={`${infoset.key}-${action.label}`}>
                            <span>{action.label}</span>
                            <strong>{formatPercent(action.probability)}</strong>
                          </li>
                        ))}
                      </ul>
                    </article>
                  ))}
                </div>
              </>
            ) : (
              <p className="placeholder">Solve a validated request to inspect root EV, exploitability, and root infoset strategies.</p>
            )}
          </section>
        </section>
      </section>
    </main>
  );
}
