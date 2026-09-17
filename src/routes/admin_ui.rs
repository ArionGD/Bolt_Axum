use axum::{
    response::Html,
};

pub async fn admin_panel() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Trisha Motors — Superuser Admin Panel (Backend Integrated)</title>
  <script src="https://cdn.tailwindcss.com"></script>
  <script src="https://unpkg.com/lucide@latest"></script>
  <script>
    tailwind.config = {
      darkMode: 'class',
      theme: {
        extend: {
          colors: {
            brand: {
              400: '#34d399',
              500: '#10b981',
              600: '#059669',
              700: '#047857',
            }
          }
        }
      }
    }
  </script>
  <style>
    body { background-color: #030712; color: #f3f4f6; font-family: system-ui, -apple-system, sans-serif; }
    .glass-card {
      background: rgba(255, 255, 255, 0.03);
      backdrop-filter: blur(24px);
      -webkit-backdrop-filter: blur(24px);
      border: 1px solid rgba(255, 255, 255, 0.08);
      box-shadow: 0 16px 40px 0 rgba(0, 0, 0, 0.45);
      position: relative;
      overflow: hidden;
    }
    .glass-shine::after {
      content: '';
      position: absolute;
      top: 0; left: 0; right: 0;
      height: 1px;
      background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.25), transparent);
    }
  </style>
</head>
<body class="min-h-screen relative overflow-x-hidden flex flex-col">
  <!-- Radial Ambient Background Lights -->
  <div class="fixed top-0 left-1/4 w-[600px] h-[450px] bg-emerald-500/10 rounded-full blur-[140px] pointer-events-none"></div>
  <div class="fixed bottom-10 right-10 w-[550px] h-[450px] bg-sky-500/10 rounded-full blur-[150px] pointer-events-none"></div>
  <div class="fixed top-1/3 right-1/4 w-[450px] h-[350px] bg-sky-600/10 rounded-full blur-[160px] pointer-events-none"></div>

  <!-- Top Glass Bar -->
  <header class="sticky top-0 z-40 border-b border-white/[0.08] bg-slate-950/80 backdrop-blur-2xl px-6 py-3.5">
    <div class="max-w-7xl mx-auto flex items-center justify-between">
      <div class="flex items-center space-x-3">
        <div class="w-10 h-10 rounded-xl bg-gradient-to-tr from-brand-600 via-emerald-400 to-sky-400 p-[1px] shadow-lg shadow-brand-500/20">
          <div class="w-full h-full bg-slate-950 rounded-[11px] flex items-center justify-center">
            <i data-lucide="zap" class="w-5 h-5 text-brand-400"></i>
          </div>
        </div>
        <div>
          <div class="flex items-center space-x-2">
            <span class="font-extrabold text-base tracking-tight text-white">Trisha<span class="text-brand-400">Motors Admin</span></span>
            <span class="px-2 py-0.5 text-[10px] font-extrabold uppercase tracking-wider bg-sky-500/20 text-sky-300 border border-sky-500/30 rounded-full">
              Axum Backend .ENV
            </span>
          </div>
          <p class="text-[11px] text-slate-400">Trisha Motors Superuser Operations Panel</p>
        </div>
      </div>

      <div class="flex items-center space-x-3">
        <div class="flex items-center space-x-2 px-3 py-1.5 rounded-xl bg-white/[0.03] border border-white/[0.08] text-xs">
          <span class="relative flex h-2 w-2">
            <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
            <span class="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
          </span>
          <span class="font-mono text-[11px] text-slate-300" id="health-badge">Axum: Online (200 OK)</span>
        </div>
        <button onclick="logoutAdmin()" id="logout-btn" class="hidden px-3 py-1.5 rounded-xl bg-red-500/10 hover:bg-red-500/20 border border-red-500/20 text-xs font-semibold text-red-300 transition-all flex items-center space-x-1">
          <i data-lucide="log-out" class="w-3.5 h-3.5"></i>
          <span>Logout</span>
        </button>
      </div>
    </div>
  </header>

  <!-- Login Gate Container -->
  <div id="auth-gate" class="flex-1 max-w-md w-full mx-auto p-6 flex flex-col justify-center relative z-10">
    <div class="glass-card glass-shine rounded-3xl p-7 border border-white/10 shadow-2xl space-y-5">
      <div class="text-center">
        <div class="w-12 h-12 rounded-2xl bg-sky-500/10 border border-sky-500/30 mx-auto flex items-center justify-center text-sky-400 mb-3">
          <i data-lucide="shield-check" class="w-6 h-6"></i>
        </div>
        <h2 class="text-xl font-extrabold text-white">Superuser Master Login</h2>
        <p class="text-xs text-slate-400 mt-1">Enter master credentials configured in your Render .env</p>
      </div>

      <div id="login-error" class="hidden p-3 bg-red-500/10 border border-red-500/30 rounded-xl text-red-300 text-xs"></div>

      <form id="login-form" onsubmit="handleAdminLogin(event)" class="space-y-4 text-xs">
        <div>
          <label class="block text-slate-300 font-bold mb-1">Superuser Email</label>
          <input type="email" id="admin-email" required value="superuser@boltcrm.com" class="w-full p-2.5 bg-white/[0.04] border border-white/10 rounded-xl text-white font-mono text-xs focus:ring-2 focus:ring-brand-500 outline-none">
        </div>
        <div>
          <label class="block text-slate-300 font-bold mb-1">Master Password</label>
          <input type="password" id="admin-pass" required placeholder="••••••••••••" class="w-full p-2.5 bg-white/[0.04] border border-white/10 rounded-xl text-white font-mono text-xs focus:ring-2 focus:ring-brand-500 outline-none">
        </div>
        <button type="submit" class="w-full py-3 bg-gradient-to-r from-brand-600 to-emerald-500 hover:from-brand-500 hover:to-emerald-400 text-white font-bold rounded-xl shadow-lg shadow-brand-500/25 transition-all flex items-center justify-center space-x-2">
          <span>Authenticate Superuser</span>
          <i data-lucide="arrow-right" class="w-4 h-4"></i>
        </button>
      </form>
    </div>
  </div>

  <!-- Authenticated Admin Portal Container -->
  <main id="portal-content" class="hidden flex-1 max-w-7xl w-full mx-auto p-6 sm:p-8 space-y-6 relative z-10">
    <!-- Header with Action -->
    <div class="glass-card glass-shine rounded-3xl p-6 sm:p-7 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <div class="inline-flex items-center space-x-1.5 px-3 py-1 rounded-full bg-sky-500/15 border border-sky-500/30 text-sky-300 text-xs font-bold mb-2">
          <i data-lucide="sparkles" class="w-3.5 h-3.5"></i>
          <span>Integrated Axum Master Hub</span>
        </div>
        <h1 class="text-2xl font-extrabold text-white tracking-tight">Superuser Account & Platform Operations</h1>
        <p class="text-xs text-slate-400 mt-1">Host-level control over Showroom Managers, Customer accounts, and marketing alert opt-ins.</p>
      </div>

      <button onclick="openModal()" class="inline-flex items-center space-x-2 px-4 py-2.5 rounded-xl bg-gradient-to-r from-brand-600 to-emerald-500 hover:from-brand-500 hover:to-emerald-400 text-white text-xs font-bold shadow-lg shadow-brand-500/20 transition-all self-start sm:self-auto">
        <i data-lucide="user-plus" class="w-4 h-4"></i>
        <span>Provision New Manager</span>
      </button>
    </div>

    <!-- KPI Metrics -->
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      <div class="glass-card glass-shine rounded-2xl p-5">
        <div class="text-[11px] font-bold uppercase tracking-wider text-slate-400 mb-1">Total Accounts</div>
        <div class="text-3xl font-extrabold text-white" id="stat-total">0</div>
        <div class="text-[11px] text-sky-400 font-semibold mt-2">Active on database</div>
      </div>
      <div class="glass-card glass-shine rounded-2xl p-5">
        <div class="text-[11px] font-bold uppercase tracking-wider text-slate-400 mb-1">Showroom Managers</div>
        <div class="text-3xl font-extrabold text-brand-400" id="stat-managers">0</div>
        <div class="text-[11px] text-slate-400 mt-2">CRM operational staff</div>
      </div>
      <div class="glass-card glass-shine rounded-2xl p-5">
        <div class="text-[11px] font-bold uppercase tracking-wider text-slate-400 mb-1">Customer Accounts</div>
        <div class="text-3xl font-extrabold text-sky-400" id="stat-customers">0</div>
        <div class="text-[11px] text-slate-400 mt-2">Shoppers & commercial buyers</div>
      </div>
      <div class="glass-card glass-shine rounded-2xl p-5">
        <div class="text-[11px] font-bold uppercase tracking-wider text-slate-400 mb-1">Alert Subscribers</div>
        <div class="text-3xl font-extrabold text-emerald-400" id="stat-alerts">0</div>
        <div class="text-[11px] text-emerald-400 font-semibold mt-2" id="stat-alert-pct">0% opted-in</div>
      </div>
    </div>

    <!-- User Accounts Table -->
    <div class="glass-card glass-shine rounded-3xl p-6 overflow-hidden space-y-4">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div>
          <h3 class="font-extrabold text-base text-white">Platform Users Directory</h3>
          <p className="text-xs text-slate-400">Superusers (.env), Showroom Managers, and registered Customers</p>
        </div>
        <input type="text" id="search-input" oninput="filterUsers()" placeholder="Search users by name or phone..." class="px-3.5 py-2 bg-white/[0.04] border border-white/10 rounded-xl text-xs text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-brand-500 w-full sm:w-64">
      </div>

      <div class="overflow-x-auto">
        <table class="w-full text-left text-xs">
          <thead>
            <tr class="border-b border-white/[0.06] text-slate-400 text-[11px] uppercase tracking-wider font-bold">
              <th class="py-3 px-4">User Name</th>
              <th class="py-3 px-4">Role</th>
              <th class="py-3 px-4">Contact</th>
              <th class="py-3 px-4">Marketing Alerts</th>
              <th class="py-3 px-4">Status</th>
              <th class="py-3 px-4 text-right">Actions</th>
            </tr>
          </thead>
          <tbody id="users-tbody" class="divide-y divide-white/[0.04]">
            <tr><td colspan="6" class="p-8 text-center text-slate-500">Loading accounts...</td></tr>
          </tbody>
        </table>
      </div>
    </div>
  </main>

  <!-- Create Manager Modal -->
  <div id="create-modal" class="fixed inset-0 z-50 bg-slate-950/80 backdrop-blur-xl hidden items-center justify-center p-4">
    <div class="glass-card glass-shine rounded-3xl p-6 max-w-md w-full border border-white/10 shadow-2xl space-y-4">
      <div class="flex items-center justify-between pb-3 border-b border-white/[0.08]">
        <div class="flex items-center space-x-2">
          <i data-lucide="user-plus" class="w-5 h-5 text-brand-400"></i>
          <h3 class="font-extrabold text-base text-white">Provision Showroom Manager</h3>
        </div>
        <button onclick="closeModal()" class="text-slate-400 hover:text-white"><i data-lucide="x" class="w-5 h-5"></i></button>
      </div>

      <form onsubmit="handleCreateManager(event)" class="space-y-3 text-xs">
        <div>
          <label class="block text-slate-300 font-bold mb-1">Designation Name *</label>
          <input type="text" id="mgr-name" required value="Manager 1" class="w-full p-2.5 bg-white/[0.04] border border-white/10 rounded-xl text-white focus:ring-2 focus:ring-brand-500 outline-none">
        </div>
        <div>
          <label class="block text-slate-300 font-bold mb-1">Work Email *</label>
          <input type="email" id="mgr-email" required placeholder="manager1@trishamotors.com" class="w-full p-2.5 bg-white/[0.04] border border-white/10 rounded-xl text-white font-mono focus:ring-2 focus:ring-brand-500 outline-none">
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="block text-slate-300 font-bold mb-1">Phone</label>
            <input type="tel" id="mgr-phone" placeholder="+91 98000 11111" class="w-full p-2.5 bg-white/[0.04] border border-white/10 rounded-xl text-white font-mono focus:ring-2 focus:ring-brand-500 outline-none">
          </div>
          <div>
            <label class="block text-slate-300 font-bold mb-1">Initial Password</label>
            <input type="text" id="mgr-pass" value="manager123" class="w-full p-2.5 bg-white/[0.04] border border-white/10 rounded-xl text-white font-mono focus:ring-2 focus:ring-brand-500 outline-none">
          </div>
        </div>

        <div class="pt-3 flex justify-end space-x-2 border-t border-white/[0.08]">
          <button type="button" onclick="closeModal()" class="px-4 py-2 text-slate-400 hover:text-white rounded-xl font-semibold">Cancel</button>
          <button type="submit" class="px-5 py-2.5 bg-gradient-to-r from-brand-600 to-emerald-500 hover:from-brand-500 hover:to-emerald-400 text-white font-bold rounded-xl shadow-md transition-all">Create Manager</button>
        </div>
      </form>
    </div>
  </div>

  <script>
    let allAccounts = [];
    let adminToken = localStorage.getItem('volt_admin_token') || '';

    function checkAuth() {
      if (adminToken) {
        document.getElementById('auth-gate').classList.add('hidden');
        document.getElementById('portal-content').classList.remove('hidden');
        document.getElementById('logout-btn').classList.remove('hidden');
        loadUsers();
      } else {
        document.getElementById('auth-gate').classList.remove('hidden');
        document.getElementById('portal-content').classList.add('hidden');
        document.getElementById('logout-btn').classList.add('hidden');
      }
      lucide.createIcons();
    }

    async function handleAdminLogin(e) {
      e.preventDefault();
      const email = document.getElementById('admin-email').value.trim();
      const password = document.getElementById('admin-pass').value.trim();
      const errEl = document.getElementById('login-error');
      errEl.classList.add('hidden');

      try {
        const res = await fetch('/api/v1/auth/login', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ email, password })
        });
        if (!res.ok) {
          throw new Error('Invalid superuser master credentials');
        }
        const data = await res.json();
        if (data.user.role !== 'superuser' && data.user.role !== 'admin') {
          throw new Error('Only Superuser accounts can access this panel');
        }
        adminToken = data.token;
        localStorage.setItem('volt_admin_token', adminToken);
        checkAuth();
      } catch (err) {
        errEl.innerText = err.message || 'Login failed';
        errEl.classList.remove('hidden');
      }
    }

    function logoutAdmin() {
      adminToken = '';
      localStorage.removeItem('volt_admin_token');
      checkAuth();
    }

    async function loadUsers() {
      try {
        const res = await fetch('/api/v1/admin/users', {
          headers: { 'Authorization': `Bearer ${adminToken}` }
        });
        if (res.ok) {
          allAccounts = await res.json();
          renderUsers(allAccounts);
          updateStats(allAccounts);
        }
      } catch (err) {
        console.error(err);
      }
    }

    function updateStats(accounts) {
      document.getElementById('stat-total').innerText = accounts.length;
      const mgrs = accounts.filter(a => a.role === 'manager' || a.role === 'sales' || a.role === 'accounts').length;
      const custs = accounts.filter(a => a.role === 'customer').length;
      const alerts = accounts.filter(a => a.role === 'customer' && a.receive_alerts).length;

      document.getElementById('stat-managers').innerText = mgrs;
      document.getElementById('stat-customers').innerText = custs;
      document.getElementById('stat-alerts').innerText = alerts;
      const pct = custs > 0 ? Math.round((alerts / custs) * 100) : 0;
      document.getElementById('stat-alert-pct').innerText = `${pct}% opted-in`;
    }

    function renderUsers(accounts) {
      const tbody = document.getElementById('users-tbody');
      if (!accounts || accounts.length === 0) {
        tbody.innerHTML = '<tr><td colspan="6" class="p-8 text-center text-slate-500">No users found</td></tr>';
        return;
      }

      tbody.innerHTML = accounts.map(a => {
        const isSuper = a.role === 'superuser' || a.role === 'admin';
        const roleBadge = isSuper
          ? '<span class="px-2 py-0.5 rounded bg-sky-500/20 text-sky-300 border border-sky-500/30 text-[10px] font-extrabold uppercase">Superuser (.ENV)</span>'
          : a.role === 'manager' || a.role === 'sales' || a.role === 'accounts'
          ? '<span class="px-2 py-0.5 rounded bg-brand-500/20 text-brand-300 border border-brand-500/30 text-[10px] font-extrabold uppercase">Manager</span>'
          : '<span class="px-2 py-0.5 rounded bg-sky-500/20 text-sky-300 border border-sky-500/30 text-[10px] font-extrabold uppercase">Customer</span>';

        const alertBadge = a.role === 'customer'
          ? (a.receive_alerts
            ? '<span class="text-emerald-400 font-semibold">🔔 Subscribed</span>'
            : '<span class="text-slate-500">Unsubscribed</span>')
          : '<span class="text-slate-600">—</span>';

        const statusBadge = a.is_active
          ? '<span class="px-2 py-0.5 rounded bg-emerald-500/15 text-emerald-400 border border-emerald-500/25 text-[10px] font-bold">Active</span>'
          : '<span class="px-2 py-0.5 rounded bg-red-500/15 text-red-400 border border-red-500/25 text-[10px] font-bold">Inactive</span>';

        const actions = !isSuper
          ? `<button onclick="toggleUser('${a.id}', ${a.is_active})" class="px-2 py-1 bg-white/[0.04] hover:bg-white/[0.08] text-slate-300 rounded-lg text-[11px] border border-white/10 font-semibold mr-1">${a.is_active ? 'Deactivate' : 'Activate'}</button>
             ${a.role === 'manager' ? `<button onclick="deleteUser('${a.id}')" class="px-2 py-1 bg-red-500/10 hover:bg-red-500/20 text-red-400 rounded-lg text-[11px] border border-red-500/20 font-semibold">Delete</button>` : ''}`
          : '<span class="text-sky-400 font-bold text-[10px]">Protected</span>';

        return `
          <tr class="hover:bg-white/[0.02] transition-colors">
            <td class="py-3 px-4 font-bold text-white">${a.full_name}</td>
            <td class="py-3 px-4">${roleBadge}</td>
            <td class="py-3 px-4 font-mono text-slate-300">${a.email || a.phone || '—'}</td>
            <td class="py-3 px-4">${alertBadge}</td>
            <td class="py-3 px-4">${statusBadge}</td>
            <td class="py-3 px-4 text-right">${actions}</td>
          </tr>
        `;
      }).join('');
      lucide.createIcons();
    }

    function filterUsers() {
      const q = document.getElementById('search-input').value.toLowerCase().trim();
      const filtered = allAccounts.filter(a =>
        a.full_name.toLowerCase().includes(q) ||
        (a.email && a.email.toLowerCase().includes(q)) ||
        (a.phone && a.phone.includes(q))
      );
      renderUsers(filtered);
    }

    function openModal() {
      document.getElementById('create-modal').classList.remove('hidden');
      document.getElementById('create-modal').classList.add('flex');
    }

    function closeModal() {
      document.getElementById('create-modal').classList.add('hidden');
      document.getElementById('create-modal').classList.remove('flex');
    }

    async function handleCreateManager(e) {
      e.preventDefault();
      const full_name = document.getElementById('mgr-name').value.trim();
      const email = document.getElementById('mgr-email').value.trim();
      const phone = document.getElementById('mgr-phone').value.trim() || undefined;
      const password = document.getElementById('mgr-pass').value.trim() || undefined;

      try {
        const res = await fetch('/api/v1/admin/managers', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${adminToken}`
          },
          body: JSON.stringify({ full_name, email, phone, password })
        });
        if (res.ok) {
          closeModal();
          loadUsers();
        } else {
          alert('Failed to provision manager');
        }
      } catch (err) {
        alert(err.message || 'Error creating manager');
      }
    }

    async function toggleUser(id, currentStatus) {
      try {
        await fetch(`/api/v1/admin/users/${id}/status`, {
          method: 'PATCH',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${adminToken}`
          },
          body: JSON.stringify({ is_active: !currentStatus })
        });
        loadUsers();
      } catch (err) {
        alert('Failed to toggle status');
      }
    }

    async function deleteUser(id) {
      if (!confirm('Are you sure you want to delete this manager?')) return;
      try {
        await fetch(`/api/v1/admin/users/${id}`, {
          method: 'DELETE',
          headers: { 'Authorization': `Bearer ${adminToken}` }
        });
        loadUsers();
      } catch (err) {
        alert('Failed to delete manager');
      }
    }

    checkAuth();
  </script>
</body>
</html>
"#)
}
