const $ = (s, el=document) => el.querySelector(s);

const API_BASE = "http://127.0.0.1:8000";

const KEY = "pcosew_session";

const toasts = $("#toasts");

function escapeHtml(str){
  return String(str).replaceAll("&","&amp;").replaceAll("<","&lt;").replaceAll(">","&gt;")
    .replaceAll('"',"&quot;").replaceAll("'","&#039;");
}
function toast(type, title, msg){
  const t = document.createElement("div");
  t.className = `toast toast--${type}`;
  t.innerHTML = `
    <div class="toast__icon" aria-hidden="true">${type === "success" ? "✅" : type === "danger" ? "⛔" : "ℹ️"}</div>
    <div>
      <div class="toast__title">${escapeHtml(title)}</div>
      <div class="toast__msg">${escapeHtml(msg)}</div>
    </div>
  `;
  toasts.appendChild(t);
  setTimeout(()=> t.remove(), 3400);
}

function saveSession(session, remember){
  const data = JSON.stringify(session);
  if (remember) localStorage.setItem(KEY, data);
  else sessionStorage.setItem(KEY, data);
}

function togglePass(inputId){
  const el = document.getElementById(inputId);
  if (!el) return;
  el.type = el.type === "password" ? "text" : "password";
}

function year(){
  const y = document.getElementById("year");
  if (y) y.textContent = new Date().getFullYear();
}
year();

async function apiRequest(path, method, body){
  const res = await fetch(`${API_BASE}${path}`, {
    method,
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });

  const data = await res.json().catch(()=> ({}));

  if (!res.ok){
    const msg = data?.error || `Error HTTP ${res.status}`;
    throw new Error(msg);
  }
  return data;
}

/* LOGIN */
const loginForm = $("#loginForm");
if (loginForm){
  $("#toggleLoginPass")?.addEventListener("click", ()=> togglePass("loginPassword"));

  $("#forgotBtn")?.addEventListener("click", (e)=>{
    e.preventDefault();
    toast("info","Aún no", "Recuperación se agrega después.");
  });

  loginForm.addEventListener("submit", async (e)=>{
    e.preventDefault();
    const email = $("#loginEmail").value.trim();
    const pass = $("#loginPassword").value;
    const remember = $("#rememberMe")?.checked;

    if (!email || !email.includes("@") || pass.length < 6){
      toast("danger","Datos inválidos","Correo válido y contraseña ≥ 6.");
      return;
    }

    try{
      const data = await apiRequest("/api/auth/login", "POST", {
        email,
        password: pass,
      });

      // data = { token, user }
      saveSession(data, remember);
      toast("success","Bienvenido", "Sesión iniciada.");
      setTimeout(()=> location.href = "./index.html", 350);
    }catch(err){
      toast("danger","Login falló", err.message);
    }
  });
}

/* REGISTER */
const registerForm = $("#registerForm");
if (registerForm){
  $("#toggleRegPass")?.addEventListener("click", ()=> togglePass("regPassword"));

  registerForm.addEventListener("submit", async (e)=>{
    e.preventDefault();
    const name = $("#regName").value.trim();
    const role = $("#regRole").value;
    const email = $("#regEmail").value.trim();
    const pass = $("#regPassword").value;
    const conf = $("#regConfirm").value;
    const terms = $("#terms").checked;

    if (!name || !email || !email.includes("@") || pass.length < 6){
      toast("danger","Datos inválidos","Completa correctamente.");
      return;
    }
    if (pass !== conf){
      toast("danger","Contraseñas","No coinciden.");
      return;
    }
    if (!terms){
      toast("danger","Términos","Debes aceptarlos.");
      return;
    }

    try{
      const data = await apiRequest("/api/auth/register", "POST", {
        name,
        email,
        password: pass,
        role,
      });

      // Guardar sesión y entrar
      saveSession(data, true);
      toast("success","Cuenta creada","Registro exitoso.");
      setTimeout(()=> location.href = "./index.html", 350);
    }catch(err){
      toast("danger","Registro falló", err.message);
    }
  });
}
