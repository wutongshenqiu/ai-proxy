import { useEffect, useState, useCallback } from 'react';
import { providersApi } from '../services/api';
import type { Provider } from '../types';
import {
  Network,
  RefreshCw,
  CheckCircle,
  XCircle,
  ArrowRight,
} from 'lucide-react';

interface ProtocolEndpoint {
  method: string;
  path: string;
  description: string;
  stream: boolean;
}

const PROTOCOLS: {
  id: string;
  label: string;
  format: string;
  endpoints: ProtocolEndpoint[];
}[] = [
  {
    id: 'openai',
    label: 'OpenAI',
    format: 'openai',
    endpoints: [
      { method: 'POST', path: '/v1/chat/completions', description: 'Chat completions', stream: true },
      { method: 'POST', path: '/v1/responses', description: 'Responses API', stream: true },
      { method: 'GET', path: '/v1/models', description: 'List models', stream: false },
    ],
  },
  {
    id: 'claude',
    label: 'Claude (Anthropic)',
    format: 'claude',
    endpoints: [
      { method: 'POST', path: '/v1/messages', description: 'Messages API', stream: true },
    ],
  },
  {
    id: 'gemini',
    label: 'Gemini (Google)',
    format: 'gemini',
    endpoints: [
      { method: 'POST', path: '/v1beta/models/{model}:generateContent', description: 'Generate content', stream: false },
      { method: 'POST', path: '/v1beta/models/{model}:streamGenerateContent', description: 'Stream generate content', stream: true },
      { method: 'GET', path: '/v1beta/models', description: 'List models', stream: false },
    ],
  },
];

type CoverageLevel = 'native' | 'adapted' | 'none';

// Translation is supported for all pairs where both protocols are in {openai, claude, gemini}.
// A provider is native when its format matches the protocol, adapted when translation exists.
const TRANSLATION_PAIRS = new Set([
  'openai->claude', 'openai->gemini',
  'claude->openai', 'claude->gemini',
  'gemini->openai', 'gemini->claude',
]);

function getCoverage(protocol: string, providerFormat: string): CoverageLevel {
  if (protocol === providerFormat) return 'native';
  const key = `${protocol}->${providerFormat}`;
  return TRANSLATION_PAIRS.has(key) ? 'adapted' : 'none';
}

function CoverageBadge({ level }: { level: CoverageLevel }) {
  if (level === 'native') {
    return <span className="type-badge type-badge--green"><CheckCircle size={12} /> Native</span>;
  }
  if (level === 'adapted') {
    return <span className="type-badge type-badge--blue"><ArrowRight size={12} /> Adapted</span>;
  }
  return <span className="type-badge type-badge--red"><XCircle size={12} /> None</span>;
}

export default function Protocols() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  const fetchProviders = useCallback(async () => {
    try {
      const res = await providersApi.list();
      setProviders(res.data.filter((p: Provider) => !p.disabled));
    } catch (err) {
      console.error('Failed to fetch providers:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchProviders();
  }, [fetchProviders]);

  const uniqueFormats = [...new Set(providers.map((p) => p.format))];

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h2>Protocols</h2>
          <p className="page-subtitle">
            Public ingress protocols, endpoint semantics, and provider coverage
          </p>
        </div>
        <div className="page-header-actions">
          <button className="btn btn-secondary" onClick={fetchProviders}>
            <RefreshCw size={16} />
            Refresh
          </button>
        </div>
      </div>

      {/* Protocol Endpoint Reference */}
      <div className="card" style={{ marginBottom: '1.5rem' }}>
        <div className="card-header">
          <h3><Network size={18} style={{ verticalAlign: 'middle', marginRight: '0.5rem' }} />Public Endpoints</h3>
        </div>
        <div className="card-body">
          <p className="text-muted" style={{ marginBottom: '1rem' }}>
            All public inference endpoints share one canonical runtime pipeline. Requests are parsed by protocol-specific ingress adapters, routed through the capability-aware planner, and translated back via egress adapters.
          </p>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
            {PROTOCOLS.map((proto) => (
              <div key={proto.id}>
                <h4 style={{ marginBottom: '0.5rem' }}>{proto.label}</h4>
                <div className="table-wrapper">
                  <table className="table">
                    <thead>
                      <tr>
                        <th style={{ width: 80 }}>Method</th>
                        <th>Path</th>
                        <th>Description</th>
                        <th style={{ width: 90 }}>Stream</th>
                      </tr>
                    </thead>
                    <tbody>
                      {proto.endpoints.map((ep) => (
                        <tr key={ep.path}>
                          <td><span className="type-badge">{ep.method}</span></td>
                          <td className="text-mono" style={{ fontSize: '0.85rem' }}>{ep.path}</td>
                          <td>{ep.description}</td>
                          <td>{ep.stream ? <CheckCircle size={14} color="var(--success)" /> : <span className="text-muted">-</span>}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Protocol × Provider Coverage Matrix */}
      <div className="card">
        <div className="card-header">
          <h3>Protocol Coverage Matrix</h3>
        </div>
        <div className="card-body">
          {isLoading ? (
            <div className="empty-state"><p>Loading providers...</p></div>
          ) : providers.length === 0 ? (
            <div className="empty-state">
              <Network size={48} />
              <p>No active providers configured</p>
            </div>
          ) : (
            <>
              <p className="text-muted" style={{ marginBottom: '1rem' }}>
                <strong>Native</strong>: Provider speaks this protocol natively. <strong>Adapted</strong>: Request is translated through the canonical IR to reach this provider.
              </p>
              <div className="table-wrapper">
                <table className="table">
                  <thead>
                    <tr>
                      <th>Provider</th>
                      <th>Format</th>
                      {PROTOCOLS.map((p) => (
                        <th key={p.id} style={{ textAlign: 'center' }}>{p.label}</th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {providers.map((provider) => (
                      <tr key={provider.name}>
                        <td className="text-bold">{provider.name}</td>
                        <td><span className="type-badge">{provider.format}</span></td>
                        {PROTOCOLS.map((proto) => (
                          <td key={proto.id} style={{ textAlign: 'center' }}>
                            <CoverageBadge level={getCoverage(proto.format, provider.format)} />
                          </td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </>
          )}
        </div>
      </div>

      {/* Format Distribution */}
      {!isLoading && providers.length > 0 && (
        <div className="card" style={{ marginTop: '1.5rem' }}>
          <div className="card-header">
            <h3>Provider Format Distribution</h3>
          </div>
          <div className="card-body">
            <div style={{ display: 'flex', gap: '2rem', flexWrap: 'wrap' }}>
              {uniqueFormats.map((fmt) => {
                const count = providers.filter((p) => p.format === fmt).length;
                return (
                  <div key={fmt} style={{ textAlign: 'center' }}>
                    <div style={{ fontSize: '2rem', fontWeight: 700 }}>{count}</div>
                    <div className="text-muted" style={{ textTransform: 'capitalize' }}>{fmt}</div>
                  </div>
                );
              })}
              <div style={{ textAlign: 'center' }}>
                <div style={{ fontSize: '2rem', fontWeight: 700 }}>{providers.length}</div>
                <div className="text-muted">Total Active</div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
