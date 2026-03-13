import { useEffect, useState } from 'react';
import { routingApi } from '../services/api';
import type { RoutingConfig } from '../types';
import { GitBranch, Save, RotateCcw } from 'lucide-react';

const PRESETS = [
  {
    key: 'balanced',
    label: 'Balanced',
    description: 'Distribute requests evenly across providers and credentials.',
  },
  {
    key: 'stable',
    label: 'Stable',
    description: 'Always use the same provider, failover only when unhealthy.',
  },
  {
    key: 'lowest-latency',
    label: 'Lowest Latency',
    description: 'Route to the fastest responding provider.',
  },
  {
    key: 'lowest-cost',
    label: 'Lowest Cost',
    description: 'Route to the cheapest available provider.',
  },
];

export default function Routing() {
  const [config, setConfig] = useState<RoutingConfig | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState('');

  const [selectedProfile, setSelectedProfile] = useState('balanced');

  const loadConfig = (data: RoutingConfig) => {
    setConfig(data);
    setSelectedProfile(data['default-profile']);
  };

  const fetchConfig = async () => {
    try {
      const response = await routingApi.get();
      loadConfig(response.data);
    } catch (err) {
      console.error('Failed to fetch routing config:', err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchConfig();
  }, []);

  const handleSave = async () => {
    setSaving(true);
    setError('');
    setSaved(false);

    try {
      await routingApi.update({ 'default-profile': selectedProfile });
      if (config) {
        loadConfig({ ...config, 'default-profile': selectedProfile });
      }
      setSaved(true);
      setTimeout(() => setSaved(false), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update routing config');
    } finally {
      setSaving(false);
    }
  };

  const handleReset = () => {
    if (config) loadConfig(config);
  };

  if (isLoading) {
    return (
      <div className="page">
        <div className="page-header">
          <h2>Routing</h2>
        </div>
        <div className="card">
          <div className="card-body">Loading...</div>
        </div>
      </div>
    );
  }

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h2>Routing</h2>
          <p className="page-subtitle">Configure request routing profile</p>
        </div>
        <div className="page-header-actions">
          <button className="btn btn-secondary" onClick={handleReset}>
            <RotateCcw size={16} />
            Reset
          </button>
          <button
            className="btn btn-primary"
            onClick={handleSave}
            disabled={saving}
          >
            <Save size={16} />
            {saving ? 'Saving...' : saved ? 'Saved!' : 'Save Changes'}
          </button>
        </div>
      </div>

      {error && <div className="alert alert-error" style={{ marginBottom: '1.5rem' }}>{error}</div>}
      {saved && <div className="alert alert-success" style={{ marginBottom: '1.5rem' }}>Routing configuration updated successfully.</div>}

      {/* Profile Selection */}
      <div className="card">
        <div className="card-header">
          <h3>Routing Profile</h3>
        </div>
        <div className="card-body">
          <div className="strategy-grid">
            {PRESETS.map((p) => (
              <label
                key={p.key}
                className={`strategy-option ${selectedProfile === p.key ? 'strategy-option--selected' : ''}`}
              >
                <input
                  type="radio"
                  name="profile"
                  value={p.key}
                  checked={selectedProfile === p.key}
                  onChange={() => setSelectedProfile(p.key)}
                />
                <div className="strategy-option-content">
                  <div className="strategy-option-header">
                    <GitBranch size={18} />
                    <span className="strategy-option-label">{p.label}</span>
                  </div>
                  <p className="strategy-option-desc">{p.description}</p>
                </div>
              </label>
            ))}
          </div>
        </div>
      </div>

      {/* Current Profile Details */}
      {config && config.profiles[selectedProfile] && (
        <div className="card">
          <div className="card-header">
            <h3>Profile Details: {selectedProfile}</h3>
          </div>
          <div className="card-body">
            <div className="settings-form">
              <div className="form-row">
                <div className="form-group">
                  <label>Provider Strategy</label>
                  <code>{config.profiles[selectedProfile]['provider-policy'].strategy}</code>
                </div>
                <div className="form-group">
                  <label>Credential Strategy</label>
                  <code>{config.profiles[selectedProfile]['credential-policy'].strategy}</code>
                </div>
              </div>
              <div className="form-row">
                <div className="form-group">
                  <label>Credential Attempts</label>
                  <code>{config.profiles[selectedProfile].failover['credential-attempts']}</code>
                </div>
                <div className="form-group">
                  <label>Provider Attempts</label>
                  <code>{config.profiles[selectedProfile].failover['provider-attempts']}</code>
                </div>
                <div className="form-group">
                  <label>Model Attempts</label>
                  <code>{config.profiles[selectedProfile].failover['model-attempts']}</code>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
