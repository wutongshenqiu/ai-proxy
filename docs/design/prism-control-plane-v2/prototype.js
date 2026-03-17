const screenDefaults = {
  'command-center': 'incident-auth-expiry',
  'traffic-lab': 'request-cld-3471',
  'provider-atlas': 'provider-openai-prod',
  'route-studio': 'route-claude-sub',
  'change-studio': 'change-routing-diff',
};

const localizedScreenMeta = {
  en: {
    'command-center': {
      kicker: 'Prototype-first redesign',
      title: 'Command Center',
      summary: 'A runtime-first shell for traffic, auth, providers, routing, change control, and external observability evidence.',
    },
    'traffic-lab': {
      kicker: 'Global context + live filters',
      title: 'Traffic Lab',
      summary: 'Saved lenses, live request grids, and runtime, hybrid, or external evidence stay in one analytical frame.',
    },
    'provider-atlas': {
      kicker: 'Runtime truth instead of config guesses',
      title: 'Provider Atlas',
      summary: 'Providers, auth posture, coverage, and capabilities are grouped as one operator catalog.',
    },
    'route-studio': {
      kicker: 'Reasoning made visible',
      title: 'Route Studio',
      summary: 'Operators build route drafts, compare selected and rejected candidates, and review publish impact without leaving the same route workspace.',
    },
    'change-studio': {
      kicker: 'Readable rollout workflow',
      title: 'Change Studio',
      summary: 'Config registry, structured CRUD, diffs, approvals, publish, and observation live together instead of being split across scattered admin pages.',
    },
  },
  'zh-CN': {
    'command-center': {
      kicker: '先做原型，再做实现',
      title: '指挥中心',
      summary: '以运行态为中心的控制平面，把流量、鉴权、Provider、路由、变更管理与外部可观测性证据统一到一个工作台里。',
    },
    'traffic-lab': {
      kicker: '全局上下文 + 实时过滤',
      title: '流量实验室',
      summary: '把保存的筛选视角、实时请求网格、链路回放，以及运行态 / 混合 / 外部证据放进同一调试框架，而不是拆成多个页面。',
    },
    'provider-atlas': {
      kicker: '运行真相，而不是配置猜测',
      title: 'Provider 图谱',
      summary: '将 Provider、鉴权状态、覆盖范围与能力矩阵合并成一个面向运维和配置的运行目录。',
    },
    'route-studio': {
      kicker: '让路由推理可见',
      title: '路由工作台',
      summary: '把路由草稿编辑、候选解释、影响面审阅与发布联动放在同一工作台里，便于解释、调试和复现路由决策。',
    },
    'change-studio': {
      kicker: '把发布过程做成工作流',
      title: '变更工作台',
      summary: '把配置对象目录、结构化 CRUD、差异审阅、预检查、灰度发布、观察窗口与回滚条件收敛到一个配置工作流中。',
    },
  },
};

const messages = {
  en: {
    'nav.section.operate': 'Operate',
    'nav.section.design': 'Design & Route',
    'nav.section.ship': 'Ship',
    'nav.commandCenter.label': 'Command Center',
    'nav.commandCenter.meta': 'Runtime posture',
    'nav.trafficLab.label': 'Traffic Lab',
    'nav.trafficLab.meta': 'Logs, traces, lenses',
    'nav.providerAtlas.label': 'Provider Atlas',
    'nav.providerAtlas.meta': 'Coverage and auth',
    'nav.routeStudio.label': 'Route Studio',
    'nav.routeStudio.meta': 'Decision simulator',
    'nav.changeStudio.label': 'Change Studio',
    'nav.changeStudio.meta': 'Diff, rollout, observe',
    'nav.footer.label': 'Prototype principle',
    'nav.footer.body': 'One shell. One context bar. One inspector. No generic blocking modals.',
    'topbar.realtimeHealthy': 'Realtime healthy',
    'topbar.window1h': '1h window',
    'topbar.tenantAll': 'Tenant: all',
    'topbar.providerMixed': 'Provider: mixed',
    'topbar.sourceHybrid': 'Source: hybrid',
    'topbar.filters': 'Global Filters',
    'topbar.jumpToTask': 'Jump to Task',
    'callout.livePosture.label': 'Live posture',
    'callout.livePosture.meta': 'healthy requests over the last hour',
    'callout.operatorQueue.label': 'Open operator queue',
    'callout.operatorQueue.meta': 'items requiring review',
    'inspector.facts': 'Facts',
    'inspector.notes': 'Notes',
    'inspector.actions': 'Actions',
    'palette.title': 'Command Palette',
    'palette.close': 'Close',
    'palette.commandCenter.title': 'Go to Command Center',
    'palette.commandCenter.meta': 'Runtime posture, health, urgent signals',
    'palette.trafficLab.title': 'Open Traffic Lab',
    'palette.trafficLab.meta': 'Logs, traces, live filters, saved lenses',
    'palette.providerAtlas.title': 'Inspect Provider Atlas',
    'palette.providerAtlas.meta': 'Provider cards, auth posture, capability truth',
    'palette.routeStudio.title': 'Simulate a Route',
    'palette.routeStudio.meta': 'Scenario builder and candidate ladder',
    'palette.changeStudio.title': 'Review Change Queue',
    'palette.changeStudio.meta': 'Draft, review, publish, observe',
  },
  'zh-CN': {
    'nav.section.operate': '运行',
    'nav.section.design': '设计与路由',
    'nav.section.ship': '发布',
    'nav.commandCenter.label': '指挥中心',
    'nav.commandCenter.meta': '运行态总览',
    'nav.trafficLab.label': '流量实验室',
    'nav.trafficLab.meta': '日志、链路、视角',
    'nav.providerAtlas.label': 'Provider 图谱',
    'nav.providerAtlas.meta': '覆盖与鉴权',
    'nav.routeStudio.label': '路由工作台',
    'nav.routeStudio.meta': '决策模拟器',
    'nav.changeStudio.label': '变更工作台',
    'nav.changeStudio.meta': '差异、发布、观察',
    'nav.footer.label': '原型原则',
    'nav.footer.body': '一个 Shell，一个上下文栏，一个 Inspector，不再依赖通用阻塞弹窗。',
    'topbar.realtimeHealthy': '实时链路健康',
    'topbar.window1h': '最近 1 小时',
    'topbar.tenantAll': '租户：全部',
    'topbar.providerMixed': 'Provider：混合',
    'topbar.sourceHybrid': '数据源：混合',
    'topbar.filters': '全局过滤器',
    'topbar.jumpToTask': '跳转任务',
    'callout.livePosture.label': '实时态势',
    'callout.livePosture.meta': '最近一小时健康请求占比',
    'callout.operatorQueue.label': '待处理队列',
    'callout.operatorQueue.meta': '当前需要人工处理的项目',
    'inspector.facts': '事实',
    'inspector.notes': '说明',
    'inspector.actions': '动作',
    'palette.title': '命令面板',
    'palette.close': '关闭',
    'palette.commandCenter.title': '前往指挥中心',
    'palette.commandCenter.meta': '查看运行态、健康度与紧急信号',
    'palette.trafficLab.title': '打开流量实验室',
    'palette.trafficLab.meta': '查看日志、链路、实时过滤与保存视角',
    'palette.providerAtlas.title': '查看 Provider 图谱',
    'palette.providerAtlas.meta': '查看 Provider 卡片、鉴权状态与能力真相',
    'palette.routeStudio.title': '模拟一次路由',
    'palette.routeStudio.meta': '进入场景构建器与候选链路对比',
    'palette.changeStudio.title': '审阅变更队列',
    'palette.changeStudio.meta': '查看草稿、审阅、发布和观察流程',
  },
};

const inspectorRecords = {
  'incident-auth-expiry': {
    kicker: 'Needs action',
    title: 'Anthropic subscription token nearing expiry',
    summary: 'This is the kind of state that should stay visible in the shell, not buried inside provider edit forms.',
    facts: [
      ['Profile', 'claude-subscription-main'],
      ['Dependent providers', '2 production routes'],
      ['Expires in', '43 minutes'],
      ['Fallback available', 'Yes'],
    ],
    notes: [
      'The new shell treats auth posture as an operational concern, not just a settings concern.',
      'The recommended next action is to rotate inside an embedded workbench and keep this incident pinned until the watch window closes.',
    ],
    actions: ['Open Auth Rotation', 'Pause Dependent Route', 'Add 15m Watch Window'],
  },
  'incident-route-pressure': {
    kicker: 'Review routing change',
    title: 'Route pressure increased after provider weight update',
    summary: 'The workspace ties change review directly to runtime pressure and fallback behavior.',
    facts: [
      ['Model family', 'gpt-5-mini'],
      ['Changed by', 'root'],
      ['Observed effect', '+12% on fallback rate'],
      ['Blast radius', 'tenant-red, sdk-app'],
    ],
    notes: [
      'This would be reachable from both Command Center and Change Studio without changing mental model.',
      'The inspector can render the same entity from multiple entry points.',
    ],
    actions: ['Open Change Diff', 'Simulate Route', 'Rollback Draft'],
  },
  'incident-drift-review': {
    kicker: 'Draft differs from runtime',
    title: 'Config drift detected between staged draft and current runtime',
    summary: 'Change Studio should explain drift in operator language first, then allow raw config drill-down second.',
    facts: [
      ['Areas changed', 'routing, tenant limits'],
      ['Validation', 'clean'],
      ['Pending approvals', '1'],
      ['Runtime impact', 'low'],
    ],
    notes: [
      'The eventual implementation should present structured diffs before raw YAML.',
      'This also makes approvals easier for non-authors.',
    ],
    actions: ['Open Change Studio', 'View Raw Diff', 'Notify Reviewer'],
  },
  'investigation-fallback-regression': {
    kicker: 'Open investigation',
    title: 'fallback-regression-cn groups request, route, provider, and change evidence',
    summary: 'A north-star Prism should promote recurring failures into first-class investigations instead of leaving them as transient table rows.',
    facts: [
      ['Owner', 'gateway-oncall'],
      ['Status', 'watching'],
      ['Pinned evidence', 'request, route, provider, change, SLS'],
      ['Started', '12 minutes ago'],
    ],
    notes: [
      'This is intentionally beyond current backend scope and represents the product model Prism should grow into.',
      'The key point is that operators can preserve findings, compare evidence, and hand off work without rebuilding context from scratch.',
    ],
    actions: ['Open Investigation', 'Compare Time Ranges', 'Link Change Watch'],
  },
  'integration-runtime-plane': {
    kicker: 'Primary truth source',
    title: 'Prism runtime stays authoritative for live operator actions',
    summary: 'Extensibility should not dilute runtime truth. Live request actions, route explain, auth posture, and publish state still come from Prism first.',
    facts: [
      ['Source', 'Prism runtime'],
      ['Use for', 'live requests, route explain, config apply'],
      ['Latency', 'sub-second'],
      ['Action safety', 'authoritative'],
    ],
    notes: [
      'External analytics can extend history and correlation, but they should not replace runtime status for control-plane actions.',
      'This rule keeps the shell reliable even when external systems lag or disconnect.',
    ],
    actions: ['Open Traffic Lab', 'Inspect Runtime Sources', 'Pin Source Policy'],
  },
  'integration-sls-plane': {
    kicker: 'External analytics',
    title: 'SLS should plug into Traffic Lab as a typed data source',
    summary: 'SLS is best treated as long-range log analytics and correlation evidence, not as a separate mini-product inside the navigation.',
    facts: [
      ['Mode', 'hybrid or external'],
      ['Best for', 'indexed search, long-range analysis'],
      ['Correlates by', 'request id, tenant, provider, model'],
      ['Operator action source', 'still Prism runtime'],
    ],
    notes: [
      'The right pattern is Runtime / Hybrid / SLS, not a dedicated SLS page that breaks the operator flow.',
      'Deep links should preserve time range, request id, tenant, provider, and model when jumping out.',
    ],
    actions: ['Open SLS Lens', 'Copy Deep Link Template', 'Compare With Runtime'],
  },
  'integration-otel-plane': {
    kicker: 'Evidence expansion',
    title: 'OTLP traces, audit files, and warehouses extend evidence without reshaping the shell',
    summary: 'The control plane should support external traces, archived audit, and warehouse analytics as overlays, compare views, and deep links.',
    facts: [
      ['Examples', 'OTLP, Tempo, Jaeger, JSONL, ClickHouse'],
      ['Role', 'history, traces, aggregate evidence'],
      ['UI pattern', 'overlay, compare, deep link'],
      ['Navigation impact', 'none'],
    ],
    notes: [
      'Do not add one top-level nav item per integration.',
      'Source provenance should stay visible so operators always know whether evidence is runtime, native analytics, or external.',
    ],
    actions: ['Open Integration Registry', 'View Provenance Rules', 'Inspect Watch Window'],
  },
  'request-cld-3471': {
    kicker: 'Selected request',
    title: 'claude-sub-01 fallback incident',
    summary: 'Traffic detail moves into the shared inspector so the main grid stays visible during analysis.',
    facts: [
      ['Request id', 'req_cld_3471'],
      ['Tenant', 'team-red'],
      ['Latency', '3,284 ms'],
      ['Failure mode', 'upstream 503 → failover'],
    ],
    notes: [
      'The prototype replaces a dedicated drawer with one inspector that already exists at shell level.',
      'The eventual URL should encode workspace, filters, selected row, and panel target.',
    ],
    actions: ['Replay Request', 'Open Route Studio', 'Pin to Incident Queue'],
  },
  'request-oai-9021': {
    kicker: 'Selected request',
    title: 'openai-prod success trace',
    summary: 'Successful traces should be as inspectable as failures for comparison during incidents.',
    facts: [
      ['Request id', 'req_oai_9021'],
      ['Mode', 'responses'],
      ['Latency', '428 ms'],
      ['Cost', '$0.017'],
    ],
    notes: [
      'Comparative inspection is important during post-change watches.',
      'The inspector can host latency breakdown, request headers, and normalized usage fields.',
    ],
    actions: ['Compare Against Failing Route', 'Copy Trace Link', 'Create Saved Lens'],
  },
  'request-gmi-2203': {
    kicker: 'Rate limited',
    title: 'gemini-cn rate limit event',
    summary: 'The same shared inspector pattern supports quota analysis, tenant history, and retry decisions.',
    facts: [
      ['Request id', 'req_gmi_2203'],
      ['Tenant', 'mobile-red'],
      ['Status', '429'],
      ['Retry after', '30s'],
    ],
    notes: [
      'Tenant context and provider context should be reachable from the same inspector.',
      'This is where action shortcuts become useful.',
    ],
    actions: ['Open Tenant Limits', 'Pause Live Feed', 'Create Retry Watch'],
  },
  'request-cdx-7122': {
    kicker: 'Realtime websocket',
    title: 'codex-main websocket session',
    summary: 'WebSocket and long-lived traffic should feel native in the same workspace, not like an edge case.',
    facts: [
      ['Request id', 'req_cdx_7122'],
      ['Transport', 'responses ws'],
      ['Session length', '04m 12s'],
      ['Auth mode', 'managed oauth'],
    ],
    notes: [
      'Prism now has enough protocol surface that the UI must normalize complexity instead of multiplying pages.',
      'The shell should own live/paused state globally.',
    ],
    actions: ['Open Session Replay', 'Inspect Auth Profile', 'Open Provider Atlas'],
  },
  'provider-openai-prod': {
    kicker: 'Healthy provider',
    title: 'openai-prod runtime profile',
    summary: 'Provider Atlas merges provider list, model coverage, protocol support, and auth posture into one catalog.',
    facts: [
      ['Upstream', 'OpenAI'],
      ['Region', 'us-east'],
      ['Auth profiles', '6 connected'],
      ['Model count', '74'],
    ],
    notes: [
      'The current dashboard splits this into providers, models, and separate auth surfaces.',
      'The prototype keeps those views connected through one entity-centric inspector.',
    ],
    actions: ['Edit Provider', 'Inspect Auth Profiles', 'Open Route Usage'],
  },
  'provider-claude-sub': {
    kicker: 'Attention needed',
    title: 'claude-sub-01 is healthy but auth rotation is pending',
    summary: 'Health and auth readiness should sit together because one can invalidate the other.',
    facts: [
      ['Upstream', 'Claude'],
      ['Profile mode', 'subscription'],
      ['Connected profiles', '2'],
      ['Runtime health', 'degraded'],
    ],
    notes: [
      'This is a good example of why providers and auth should not be separated by route boundaries in the UI.',
      'The inspector can expose both runtime and identity facts without opening a modal.',
    ],
    actions: ['Rotate Auth', 'Open Incidents', 'Simulate Route Impact'],
  },
  'provider-gemini-cn': {
    kicker: 'Capability-led view',
    title: 'gemini-cn multimodal coverage',
    summary: 'Capability truth is easier to understand as part of provider context than as a standalone page.',
    facts: [
      ['Upstream', 'Gemini'],
      ['Images', 'supported'],
      ['Count tokens', 'supported'],
      ['Tools', 'partial'],
    ],
    notes: [
      'The catalog makes supported and unsupported surfaces visually comparable.',
      'This reduces page-hopping between protocols, models, and providers.',
    ],
    actions: ['View Models', 'Compare Capabilities', 'Create Route Rule'],
  },
  'provider-codex-main': {
    kicker: 'Specialized provider',
    title: 'codex-main websocket coverage',
    summary: 'Niche protocol and auth behaviors still fit into the same card and inspector model.',
    facts: [
      ['Upstream', 'Codex'],
      ['Transport', 'responses websocket'],
      ['Auth', 'managed oauth'],
      ['Model count', '4'],
    ],
    notes: [
      'A unified shell needs to handle rare surfaces without inventing a separate mini-product.',
      'The inspector should support protocol-specific facts when needed.',
    ],
    actions: ['Open WS Metrics', 'Manage OAuth', 'Inspect Replay'],
  },
  'route-claude-sub': {
    kicker: 'Planner selected',
    title: 'claude-sub-01 chosen for reasoning-heavy messages traffic',
    summary: 'This screen keeps the builder, route ladder, and result detail visible in one place.',
    facts: [
      ['Execution mode', 'native'],
      ['Reason chosen', 'messages + reasoning + cn region'],
      ['Fallback available', 'yes'],
      ['Estimated latency', '680 ms'],
    ],
    notes: [
      'The prototype avoids hidden score fields and opaque picker behavior.',
      'Every selected candidate should explain itself with the same truth source as runtime dispatch.',
    ],
    actions: ['Replay with Same Context', 'Inspect Provider', 'Copy Explain URL'],
  },
  'route-openai-adapted': {
    kicker: 'Rejected candidate',
    title: 'openai-prod rejected due to reasoning parity mismatch',
    summary: 'Rejected routes should be inspectable, not collapsed into invisible backend logic.',
    facts: [
      ['Execution mode', 'adapted'],
      ['Reject reason', 'capability mismatch'],
      ['Would be lossy', 'yes'],
      ['Region', 'us-east'],
    ],
    notes: [
      'Visible rejections reduce operator confusion during incident review.',
      'They also make route policy debugging much faster.',
    ],
    actions: ['Compare with Winner', 'Edit Route Rule', 'Inspect Capability Matrix'],
  },
  'route-gemini-reject': {
    kicker: 'Rejected candidate',
    title: 'gemini-cn rejected because messages surface is unavailable',
    summary: 'The UI should expose protocol-surface constraints in operator language.',
    facts: [
      ['Reject type', 'unsupported surface'],
      ['Required protocol', 'messages'],
      ['Available', 'generateContent only'],
      ['Lossless fallback', 'not possible'],
    ],
    notes: [
      'This is exactly the sort of detail that benefits from a consistent inspector and route workspace.',
      'The prototype treats unsupported states as first-class signals, not hidden implementation trivia.',
    ],
    actions: ['Open Protocol Matrix', 'Change Ingress', 'Copy Reject Reason'],
  },
  'route-draft-profile': {
    kicker: 'Route draft object',
    title: 'reasoning-cn-default-v2 is a publishable route object, not a loose rules patch',
    summary: 'The route draft should preserve identity, baseline origin, intended scenarios, and rollback anchor so operators can reason about it as a managed object.',
    facts: [
      ['Draft id', 'reasoning-cn-default-v2'],
      ['Based on', 'balanced'],
      ['Intent', 'prioritize Claude for cn reasoning traffic'],
      ['Rollback anchor', 'profile v18'],
    ],
    notes: [
      'This is the route equivalent of object-based config management in Change Studio.',
      'Turning route edits into named draft objects makes review, rollback, and collaboration much clearer.',
    ],
    actions: ['Edit Draft Shell', 'Compare Baseline', 'Open Change Linkage'],
  },
  'route-draft-matchers': {
    kicker: 'Route matcher set',
    title: 'Route matchers should describe traffic intent in operator language',
    summary: 'A good route builder makes matchers readable: region, reasoning need, ingress surface, and auth posture should map cleanly to expected targets.',
    facts: [
      ['Primary matcher', 'region=cn && reasoning=true && ingress=messages'],
      ['Expected winner', 'claude-sub-01'],
      ['Fallback path', 'openai-prod'],
      ['Scenario count', '3 reviewed'],
    ],
    notes: [
      'Operators need to understand which traffic shape they are moving before they publish the rule.',
      'This is where scenario-led editing is much stronger than a raw rule table alone.',
    ],
    actions: ['Edit Matchers', 'Run Scenario Matrix', 'Inspect Fallback Logic'],
  },
  'route-draft-preview': {
    kicker: 'Simulation result',
    title: 'Preview and explain should validate the route draft before publish',
    summary: 'The route builder should show the chosen candidate, rejected alternatives, and explain output while the draft is still being edited.',
    facts: [
      ['Winner', 'claude-sub-01'],
      ['Fallback', 'openai-prod'],
      ['Explain depth', 'full scoring available'],
      ['Baseline compare', 'enabled'],
    ],
    notes: [
      'This keeps route editing attached to runtime truth rather than speculative config editing.',
      'Preview and explain are validation gates, not optional extras.',
    ],
    actions: ['Run Full Explain', 'Compare Against Current', 'Copy Replay URL'],
  },
  'route-draft-impact': {
    kicker: 'Blast radius review',
    title: 'Route draft impact should be explicit before publish',
    summary: 'Before promoting a route change, operators should see tenants, dependent providers, active investigations, and rollback anchors in one place.',
    facts: [
      ['Affected tenants', '7'],
      ['Providers touched', '2'],
      ['Linked investigations', '1'],
      ['Watch windows', '1 active'],
    ],
    notes: [
      'This is the minimum bar for safe routing changes in a serious gateway control plane.',
      'Impact review belongs in Route Studio, not only in a later publish dialog.',
    ],
    actions: ['Inspect Dependents', 'Open Linked Investigation', 'Review Rollback Target'],
  },
  'route-draft-publish': {
    kicker: 'Publish linkage',
    title: 'Route drafts should flow directly into Change Studio rollout',
    summary: 'Route Studio should not stop at “save”. A mature route editor hands its draft to review, canary, observation, and rollback workflow immediately.',
    facts: [
      ['Change type', 'routing'],
      ['Canary path', '5% -> 25% -> 100%'],
      ['Watch window', '15 minutes'],
      ['Observation source', 'Prism live + SLS'],
    ],
    notes: [
      'This ties route design to operational safety rather than treating them as separate products.',
      'It also prevents the route builder from becoming another isolated admin page.',
    ],
    actions: ['Open Change Studio Draft', 'Attach Watch Window', 'Schedule Canary'],
  },
  'change-routing-diff': {
    kicker: 'Pending publish',
    title: 'Routing diff prioritizes Claude for reasoning-heavy cn traffic',
    summary: 'Change Studio turns raw config into readable intent, predicted effect, and observation plan.',
    facts: [
      ['Area', 'routing'],
      ['Risk', 'low'],
      ['Needs approval', '1 reviewer'],
      ['Watch window', '15 minutes'],
    ],
    notes: [
      'Readable diffs are critical when operators are tired or not the original author.',
      'Raw YAML should remain available, but it should not be the default surface.',
    ],
    actions: ['Approve Draft', 'Open Raw Diff', 'Attach Watch Window'],
  },
  'change-auth-rotation': {
    kicker: 'High-value change',
    title: 'Auth rotation removes imminent subscription outage risk',
    summary: 'Identity changes belong in the same change queue when they affect runtime behavior.',
    facts: [
      ['Area', 'auth'],
      ['Profiles affected', '2'],
      ['Expected downtime', 'none'],
      ['Rollback', 'available'],
    ],
    notes: [
      'The prototype intentionally blurs the boundary between auth and runtime operations.',
      'That is where operators actually experience these changes.',
    ],
    actions: ['Review Rotation', 'Inspect Provider Dependents', 'Publish Now'],
  },
  'change-tenant-limit': {
    kicker: 'Temporary policy',
    title: 'Tenant limit override with automatic rollback reminder',
    summary: 'Temporary control-plane changes need expiration and observation baked into the workflow.',
    facts: [
      ['Area', 'tenant policy'],
      ['Duration', '24 hours'],
      ['Affected tenant', 'team-red'],
      ['Rollback plan', 'scheduled'],
    ],
    notes: [
      'This is a better interaction model than a quick modal because it preserves context and accountability.',
      'Observation belongs beside publication, not on a separate system page.',
    ],
    actions: ['Approve Temporary Override', 'Add Expiry Reminder', 'Open Tenant History'],
  },
  'crud-providers-registry': {
    kicker: 'Config object family',
    title: 'Providers should support full structured lifecycle management',
    summary: 'A rich control plane cannot stop at list and edit. Providers need query, create, clone, disable, guarded delete, dependency checks, and version history.',
    facts: [
      ['Objects', '14 providers'],
      ['Core verbs', 'query, create, edit, clone, disable, delete'],
      ['Special actions', 'health, model fetch, preview, dependency graph'],
      ['Delete mode', 'guarded by impact analysis'],
    ],
    notes: [
      'Provider management is one of the strongest examples where CRUD is not enough by itself; runtime diagnostics and dependency visibility need to live beside it.',
      'The best-practice direction is structured forms first, raw config second, with clear blast-radius review before destructive actions.',
    ],
    actions: ['Create Provider', 'Clone Existing', 'Inspect Dependents'],
  },
  'crud-auth-profiles-registry': {
    kicker: 'Config object family',
    title: 'Auth Profiles need both CRUD and runtime lifecycle actions',
    summary: 'This object family mixes config verbs with runtime verbs: connect, import, rotate, refresh, suspend, and retire.',
    facts: [
      ['Objects', '25 auth profiles'],
      ['Core verbs', 'query, create, edit, disable, delete'],
      ['Runtime verbs', 'connect, import, rotate, refresh'],
      ['Safety', 'secret-aware and dependency-aware'],
    ],
    notes: [
      'Managed auth should never feel like a bolt-on settings page.',
      'A strong control plane makes auth identity an equal citizen beside providers and routes.',
    ],
    actions: ['Create Auth Profile', 'Rotate Credentials', 'Inspect Attached Providers'],
  },
  'crud-route-registry': {
    kicker: 'Config object family',
    title: 'Route Profiles and Rules should expose simulation-led CRUD',
    summary: 'Operators should be able to query, create, clone, archive, and rollback route policies while keeping route explain and historical evidence close by.',
    facts: [
      ['Objects', '11 profiles / 24 rules'],
      ['Core verbs', 'query, create, edit, clone, archive'],
      ['Verification', 'simulation, explain, compare'],
      ['Rollback', 'versioned'],
    ],
    notes: [
      'Best practice here is not a raw list of YAML snippets.',
      'The right unit is a route object with scenarios, dependencies, and rollback history.',
    ],
    actions: ['Create Route Profile', 'Simulate Change', 'Rollback Version'],
  },
  'crud-policy-registry': {
    kicker: 'Config object family',
    title: 'Policies, keys, sources, and alerts need registry-level management',
    summary: 'A future-facing Change Studio should also govern tenant policies, auth keys, data sources, alert rules, and watch windows.',
    facts: [
      ['Objects', '62 active records'],
      ['Core verbs', 'query, create, bulk edit, suspend, delete'],
      ['Examples', 'auth keys, tenant policies, SLS sources, alerts'],
      ['Audit', 'required'],
    ],
    notes: [
      'This is where Prism stops looking like a single-purpose config page and becomes a true control plane.',
      'Not every object needs hard delete by default; suspend and archive are often the safer first-class verbs.',
    ],
    actions: ['Open Policy Registry', 'Bulk Edit Scope', 'Inspect Audit Trail'],
  },
  'crud-openai-prod-detail': {
    kicker: 'Selected config object',
    title: 'openai-prod behaves like a managed record, not a loose YAML fragment',
    summary: 'A serious control plane should let operators inspect dependencies, runtime posture, version history, and safe lifecycle actions from the same object surface.',
    facts: [
      ['Family', 'provider'],
      ['Dependents', '12 routes / 7 tenants'],
      ['Current state', 'enabled'],
      ['Latest version', 'v43'],
    ],
    notes: [
      'This object view intentionally mixes config detail with operational context because that is how gateway changes are actually evaluated.',
      'The right mental model is a registry record with lifecycle, not a text blob with save buttons.',
    ],
    actions: ['Edit Structured Form', 'Open Version History', 'Review Delete Guard'],
  },
  'crud-route-profile-detail': {
    kicker: 'Selected config object',
    title: 'reasoning-cn-default should expose scenarios, dependencies, and rollback path',
    summary: 'Route profiles deserve their own object surface with simulation, explain, and historical comparison built in.',
    facts: [
      ['Family', 'route profile'],
      ['Attached rules', '8'],
      ['Draft status', 'ready for review'],
      ['Rollback target', 'v18'],
    ],
    notes: [
      'This is a good example of why plain CRUD is not enough for gateway routing objects.',
      'Operators need to know what traffic will move, not only what fields changed.',
    ],
    actions: ['Simulate Route', 'Compare Baseline', 'Open Rollback'],
  },
  'crud-auth-profile-detail': {
    kicker: 'Selected config object',
    title: 'anthropic-subscription-main mixes config CRUD with runtime credential lifecycle',
    summary: 'Auth profiles need standard create and edit verbs, plus connect, rotate, refresh, suspend, and dependency-aware retirement.',
    facts: [
      ['Family', 'auth profile'],
      ['Dependents', '2 providers'],
      ['Health', 'expiry warning'],
      ['Rotation audit', 'available'],
    ],
    notes: [
      'Identity is not a minor settings concern in a gateway control plane.',
      'Treating auth as a first-class object keeps risk visible before it becomes an outage.',
    ],
    actions: ['Rotate Credentials', 'Inspect Dependents', 'Open Audit Trail'],
  },
  'crud-data-source-detail': {
    kicker: 'Selected config object',
    title: 'sls-cn-runtime should be governed like a typed integration record',
    summary: 'External sources such as SLS belong in the registry so operators can test, disable, audit, and compare them without inventing a separate mini-console.',
    facts: [
      ['Family', 'data source'],
      ['Mode', 'hybrid evidence'],
      ['Health', 'connected'],
      ['Linked lenses', '3'],
    ],
    notes: [
      'This is one way to keep extensibility visible without turning integrations into new top-level product silos.',
      'A typed registry record also makes ownership and change history clearer.',
    ],
    actions: ['Test Connection', 'Open Traffic Lens', 'Inspect Audit Trail'],
  },
  'crud-create-provider-template': {
    kicker: 'Create pattern',
    title: 'New config records should start from templates, clones, or imports',
    summary: 'Creation speed matters, but so does consistency. The best control planes offer templates, clone flows, and guided wizards for high-risk object types.',
    facts: [
      ['Primary paths', 'template, clone, import'],
      ['Risk controls', 'inline validation, defaults, naming rules'],
      ['High-risk objects', 'providers, auth profiles, routes'],
      ['Output', 'draft change record'],
    ],
    notes: [
      'Creation should not dump the operator into a blank YAML editor unless they explicitly ask for it.',
      'Templates and clones also reduce accidental divergence between similar runtime objects.',
    ],
    actions: ['Start From Template', 'Clone Existing Record', 'Import External Definition'],
  },
  'crud-clone-provider-canary': {
    kicker: 'Clone pattern',
    title: 'Cloning is the fastest safe path for canaries and staged experiments',
    summary: 'Clone flows should preserve known-good fields, highlight intentional deltas, and immediately attach the new record to a draft change review.',
    facts: [
      ['Source', 'openai-prod'],
      ['Suggested target', 'openai-prod-canary'],
      ['Preserve', 'auth, timeout, proxy, presentation'],
      ['Review step', 'semantic diff before publish'],
    ],
    notes: [
      'Many gateway changes start as controlled variants of an existing object, not as brand-new records.',
      'A dedicated clone path is much safer than hand-copying config snippets.',
    ],
    actions: ['Create Canary Draft', 'Compare Fields', 'Attach Rollout Plan'],
  },
  'crud-disable-provider-guard': {
    kicker: 'Retire pattern',
    title: 'Disable and archive should be first-class before hard delete',
    summary: 'When runtime still depends on an object, the default path should be disable or retire with a watch window, not immediate deletion.',
    facts: [
      ['Preferred path', 'disable -> observe -> archive'],
      ['Blocking dependents', '12 routes'],
      ['Watch window', 'required'],
      ['Rollback', 'instant re-enable'],
    ],
    notes: [
      'Soft retirement preserves recovery options when an operator is working under pressure.',
      'This pattern is especially important for providers, auth profiles, routes, and external integrations.',
    ],
    actions: ['Plan Cutover', 'Open Watch Window', 'Archive After Stabilization'],
  },
  'crud-delete-provider-guard': {
    kicker: 'Destructive action guard',
    title: 'Hard delete should require dependency proof, audit note, and rollback confidence',
    summary: 'Delete is a valid operator verb, but it should only appear after the system proves that traffic, identity, and investigation links are safe to sever.',
    facts: [
      ['Dependencies resolved', 'not yet'],
      ['Required approvals', '1 owner + 1 reviewer'],
      ['Audit note', 'mandatory'],
      ['Rollback target', 'must exist'],
    ],
    notes: [
      'The safest control planes make destructive actions explicit instead of hiding them behind ambiguous trash icons.',
      'Showing blast radius and unresolved dependents before delete reduces accidental outages.',
    ],
    actions: ['View Dependency Proof', 'Add Audit Reason', 'Schedule Hard Delete'],
  },
  'editor-provider-pattern': {
    kicker: 'Family-specific editor',
    title: 'Provider editing should combine structure, discovery, and runtime validation',
    summary: 'The current Prism surface already has model fetch, health check, and presentation preview. The north-star editor should unify those capabilities instead of scattering them across a modal and separate pages.',
    facts: [
      ['Flow', 'identity -> connectivity -> presentation -> validate'],
      ['Runtime checks', 'model fetch, health, preview'],
      ['Secret handling', 'auth profile aware'],
      ['Output', 'draft change record'],
    ],
    notes: [
      'This editor is intentionally more workflow-shaped than the current provider modal.',
      'The key product improvement is that validation happens in context before publish, not as an afterthought.',
    ],
    actions: ['Open Structured Form', 'Link Auth Profile', 'Attach Change Review'],
  },
  'editor-provider-validate': {
    kicker: 'Runtime validation',
    title: 'Provider validation should run health, model discovery, and presentation preview together',
    summary: 'Operators should not have to jump between disconnected controls to know whether a provider record is safe to publish.',
    facts: [
      ['Checks', 'health, model fetch, presentation preview'],
      ['Protected headers', 'blocked visibly'],
      ['Failure output', 'operator-readable'],
      ['Publish gate', 'required for risky changes'],
    ],
    notes: [
      'This keeps provider setup aligned with runtime truth rather than trusting static form completion.',
      'The UI should degrade to unknown when runtime proof is unavailable.',
    ],
    actions: ['Run Full Validation', 'Inspect Failure Detail', 'Pin Watch Window'],
  },
  'editor-auth-wizard': {
    kicker: 'Family-specific editor',
    title: 'Auth profiles need a wizard that respects both config and runtime lifecycle',
    summary: 'Auth records mix stable metadata with volatile credentials. The editor should therefore separate create-shell, choose-mode, connect, and verify steps.',
    facts: [
      ['Modes', 'api key, oauth, subscription'],
      ['Connect paths', 'paste, import, browser, device'],
      ['Runtime proof', 'connected, expires, attached providers'],
      ['Post-create action', 'rotate or refresh'],
    ],
    notes: [
      'This is more explicit than the current page, but it better matches the real operator task.',
      'Managed auth should feel like a first-class workflow, not a small settings branch.',
    ],
    actions: ['Create Empty Profile', 'Pick Connect Method', 'Verify Runtime Identity'],
  },
  'editor-auth-connect': {
    kicker: 'Connect flow',
    title: 'Choose the connection path based on auth mode and operator context',
    summary: 'OAuth, device flow, token paste, and server-local import should be presented as distinct, understandable paths with runtime preconditions visible.',
    facts: [
      ['OAuth', 'browser redirect'],
      ['Device flow', 'server egress dependent'],
      ['Import', 'server-local auth bundle'],
      ['Subscription', 'paste setup token'],
    ],
    notes: [
      'The user should understand where the credential actually lives and what network path the runtime will use.',
      'This reduces confusion when auth succeeds in the browser but fails from Prism server egress.',
    ],
    actions: ['Open Browser OAuth', 'Start Device Flow', 'Import Local Bundle'],
  },
  'editor-route-builder': {
    kicker: 'Family-specific editor',
    title: 'Route editing should stay scenario-led and simulation-first',
    summary: 'Prism already has preset cards, a rule table, advanced profile editing, and route preview. The north-star builder should combine them into one progressive workbench.',
    facts: [
      ['Flow', 'profile -> matchers -> simulate -> impact'],
      ['Runtime source', 'same planner as runtime'],
      ['Comparison', 'winner + rejected candidates'],
      ['Publish link', 'draft change'],
    ],
    notes: [
      'The point is not to hide advanced controls, but to stop forcing operators to mentally stitch multiple widgets together.',
      'Route objects should feel testable before they are publishable.',
    ],
    actions: ['Edit Matchers', 'Simulate Decision', 'Review Blast Radius'],
  },
  'editor-route-simulate': {
    kicker: 'Simulation gate',
    title: 'Route preview and explain should be mandatory companions to route edits',
    summary: 'Before publishing a route change, operators should be able to preview the selected candidate, the rejected candidates, and the reasons behind both.',
    facts: [
      ['Preview level', 'lightweight plan'],
      ['Explain level', 'full scoring detail'],
      ['Impact view', 'tenants, providers, investigations'],
      ['Rollback target', 'previous version'],
    ],
    notes: [
      'This is the route equivalent of provider health validation.',
      'Simulation-led editing is one of the clearest best-practice upgrades over flat admin forms.',
    ],
    actions: ['Run Preview', 'Open Full Explain', 'Compare Against Baseline'],
  },
};

const appState = {
  locale: 'en',
  screen: 'command-center',
  inspector: screenDefaults['command-center'],
};

const screenButtons = [...document.querySelectorAll('[data-screen-target]')];
const localeButtons = [...document.querySelectorAll('[data-locale]')];
const screens = [...document.querySelectorAll('.screen')];
const titleNode = document.getElementById('workspace-title');
const kickerNode = document.getElementById('workspace-kicker');
const summaryNode = document.getElementById('workspace-summary');
const inspectorKickerNode = document.getElementById('inspector-kicker');
const inspectorTitleNode = document.getElementById('inspector-title');
const inspectorSummaryNode = document.getElementById('inspector-summary');
const factsNode = document.getElementById('inspector-facts');
const notesNode = document.getElementById('inspector-notes');
const actionsNode = document.getElementById('inspector-actions');
const palette = document.getElementById('command-palette');
const paletteBackdrop = document.getElementById('palette-backdrop');
const openPaletteButton = document.getElementById('open-command-palette');
const closePaletteButton = document.getElementById('close-command-palette');
const urlState = new URLSearchParams(window.location.search);

function t(key) {
  return messages[appState.locale]?.[key] ?? messages.en[key] ?? key;
}

function syncUrlState() {
  const params = new URLSearchParams(window.location.search);
  params.set('screen', appState.screen);
  params.set('locale', appState.locale);
  params.set('inspector', appState.inspector);
  window.history.replaceState({}, '', `${window.location.pathname}?${params.toString()}`);
}

function getScreenCopy(screenId) {
  return localizedScreenMeta[appState.locale]?.[screenId] ?? localizedScreenMeta.en[screenId];
}

function renderWorkspaceCopy() {
  const meta = getScreenCopy(appState.screen);
  if (!meta) return;
  kickerNode.textContent = meta.kicker;
  titleNode.textContent = meta.title;
  summaryNode.textContent = meta.summary;
}

function renderInspector(recordId) {
  const record = inspectorRecords[recordId];
  if (!record) return;

  appState.inspector = recordId;
  inspectorKickerNode.textContent = record.kicker;
  inspectorTitleNode.textContent = record.title;
  inspectorSummaryNode.textContent = record.summary;

  factsNode.innerHTML = record.facts
    .map(
      ([label, value]) =>
        `<div class="fact-row"><span>${label}</span><strong>${value}</strong></div>`,
    )
    .join('');

  notesNode.innerHTML = record.notes
    .map((note) => `<div class="note-item">${note}</div>`)
    .join('');

  actionsNode.innerHTML = record.actions
    .map((action) => `<button type="button">${action}</button>`)
    .join('');

  syncUrlState();
}

function applyLocale(locale) {
  appState.locale = locale;
  document.documentElement.lang = locale;

  document.querySelectorAll('[data-i18n]').forEach((node) => {
    const key = node.getAttribute('data-i18n');
    if (!key) return;
    node.textContent = t(key);
  });

  localeButtons.forEach((button) => {
    button.classList.toggle('locale-pill--active', button.dataset.locale === locale);
  });

  renderWorkspaceCopy();
  syncUrlState();
}

function setScreen(screenId) {
  const meta = getScreenCopy(screenId);
  if (!meta) return;

  appState.screen = screenId;

  screenButtons.forEach((button) => {
    const matches = button.dataset.screenTarget === screenId;
    button.classList.toggle('nav-item--active', matches && button.classList.contains('nav-item'));
  });

  screens.forEach((screen) => {
    screen.classList.toggle('screen--active', screen.dataset.screen === screenId);
  });

  renderWorkspaceCopy();
  renderInspector(screenDefaults[screenId]);
  syncUrlState();
}

function togglePalette(nextOpen) {
  const open = nextOpen ?? palette.hasAttribute('hidden');
  palette.toggleAttribute('hidden', !open);
  paletteBackdrop.toggleAttribute('hidden', !open);
}

screenButtons.forEach((button) => {
  button.addEventListener('click', () => {
    const screenId = button.dataset.screenTarget;
    if (screenId) {
      setScreen(screenId);
      togglePalette(false);
    }
  });
});

localeButtons.forEach((button) => {
  button.addEventListener('click', () => {
    const locale = button.dataset.locale;
    if (locale) {
      applyLocale(locale);
    }
  });
});

document.querySelectorAll('[data-inspector-id]').forEach((element) => {
  element.addEventListener('click', () => {
    const recordId = element.getAttribute('data-inspector-id');
    if (!recordId) return;

    const siblings = element.parentElement?.querySelectorAll('.is-selected, .action-row--active, .provider-card--selected, .diff-card--selected');
    siblings?.forEach((node) => {
      node.classList.remove('is-selected', 'action-row--active', 'provider-card--selected', 'diff-card--selected');
    });

    if (element.tagName === 'TR') {
      element.classList.add('is-selected');
    }
    if (element.classList.contains('action-row')) {
      element.classList.add('action-row--active');
    }
    if (element.classList.contains('provider-card')) {
      element.classList.add('provider-card--selected');
    }
    if (element.classList.contains('diff-card')) {
      element.classList.add('diff-card--selected');
    }

    renderInspector(recordId);
  });
});

openPaletteButton.addEventListener('click', () => togglePalette(true));
closePaletteButton.addEventListener('click', () => togglePalette(false));
paletteBackdrop.addEventListener('click', () => togglePalette(false));

document.addEventListener('keydown', (event) => {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
    event.preventDefault();
    togglePalette(true);
  }
  if (event.key === 'Escape') {
    togglePalette(false);
  }
});

const initialLocale = urlState.get('locale');
if (initialLocale && messages[initialLocale]) {
  appState.locale = initialLocale;
}

const initialScreen = urlState.get('screen');
if (initialScreen && screenDefaults[initialScreen]) {
  appState.screen = initialScreen;
  appState.inspector = screenDefaults[initialScreen];
}

applyLocale(appState.locale);
setScreen(appState.screen);

const initialInspector = urlState.get('inspector');
if (initialInspector && inspectorRecords[initialInspector]) {
  renderInspector(initialInspector);
}
