const $ = (s, el=document) => el.querySelector(s);
const $$ = (s, el=document) => [...el.querySelectorAll(s)];

const KEY = "pcosew_session";
const API_BASE = "http://127.0.0.1:8000";

function getSession(){
  const a = localStorage.getItem(KEY);
  const b = sessionStorage.getItem(KEY);
  const raw = a || b;
  if (!raw) return null;
  try { return JSON.parse(raw); } catch { return null; }
}
function clearSession(){
  localStorage.removeItem(KEY);
  sessionStorage.removeItem(KEY);
}

function escapeHtml(str){
  return String(str).replaceAll("&","&amp;").replaceAll("<","&lt;").replaceAll(">","&gt;")
    .replaceAll('"',"&quot;").replaceAll("'","&#039;");
}
function toast(type, title, msg){
  const toasts = $("#toasts");
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
  setTimeout(()=> t.remove(), 3200);
}

function extOf(name){ const i = name.lastIndexOf("."); return i >= 0 ? name.slice(i+1).toLowerCase() : "file"; }
function iconFor(ext){
  const map = { pdf:"📄", png:"🖼️", jpg:"🖼️", jpeg:"🖼️", zip:"🗜️", txt:"📝", md:"📝", json:"🧩" };
  return map[ext] || "📦";
}
function formatBytes(bytes){
  if (!bytes || bytes === 0) return "0 B";
  const k=1024, sizes=["B","KB","MB","GB"];
  const i = Math.floor(Math.log(bytes)/Math.log(k));
  const val = bytes/Math.pow(k,i);
  return `${val.toFixed(val >= 10 || i === 0 ? 0 : 1)} ${sizes[i]}`;
}
function formatDate(iso){
  const d = new Date(iso);
  const pad = (n)=> String(n).padStart(2,"0");
  return `${pad(d.getDate())}/${pad(d.getMonth()+1)}/${d.getFullYear()} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}
function avatarFromName(name="U"){
  const parts = String(name).trim().split(/\s+/).slice(0,2);
  return parts.map(p=> p[0]?.toUpperCase() || "").join("") || "U";
}

/* Protect */
const session = getSession();
if (!session?.token){
  location.href = "./login.html";
}

/* Token */
const token = session?.token;

/* API helpers */
async function apiJson(path, method="GET", body){
  const res = await fetch(`${API_BASE}${path}`, {
    method,
    headers: {
      "Authorization": `Bearer ${token}`,
      ...(body ? { "Content-Type":"application/json" } : {})
    },
    body: body ? JSON.stringify(body) : undefined
  });

  const data = await res.json().catch(()=> ({}));
  if (!res.ok){
    throw new Error(data?.error || `HTTP ${res.status}`);
  }
  return data;
}

async function apiUpload(file){
  const fd = new FormData();
  fd.append("file", file);

  const res = await fetch(`${API_BASE}/api/files/upload`, {
    method: "POST",
    headers: { "Authorization": `Bearer ${token}` },
    body: fd
  });

  const data = await res.json().catch(()=> ({}));
  if (!res.ok){
    throw new Error(data?.error || `HTTP ${res.status}`);
  }
  return data;
}

/* State */
const state = { files: [], filter:"all", sort:"newest", search:"" };

/* Elements */
const els = {
  appView: $("#appView"),
  sidebar: $(".sidebar"),
  btnSidebar: $("#btnSidebar"),
  userAvatar: $("#userAvatar"),
  userName: $("#userName"),
  userRole: $("#userRole"),
  btnLogout: $("#btnLogout"),

  searchInput: $("#searchInput"),
  sortSelect: $("#sortSelect"),
  tbody: $("#filesTbody"),
  empty: $("#emptyState"),
  chkAll: $("#chkAll"),
  btnBulkDelete: $("#btnBulkDelete"),
  btnRefresh: $("#btnRefresh"),

  statFiles: $("#statFiles"),
  statStorage: $("#statStorage"),
  statPublic: $("#statPublic"),
  statDownloads: $("#statDownloads"),

  modal: $("#uploadModal"),
  btnOpenUpload: $("#btnOpenUpload"),
  btnEmptyUpload: $("#btnEmptyUpload"),
  uploadForm: $("#uploadForm"),
  fileInput: $("#fileInput"),
  dropzone: $("#dropzone"),
  filePicked: $("#filePicked"),
  pickedName: $("#pickedName"),
  pickedMeta: $("#pickedMeta"),
  customName: $("#customName"),            // (solo UI, backend usa original_name)
  visibilitySelect: $("#visibilitySelect") // (backend por ahora sube privado, luego lo seteamos)
};

/* Load files from backend */
async function loadFiles(){
  try{
    const list = await apiJson("/api/files", "GET");

    state.files = list.map(f => ({
      id: f.id,
      name: f.original_name,
      originalName: f.original_name,
      ext: extOf(f.original_name),
      size: f.size,
      visibility: f.visibility,
      updatedAt: f.updated_at,
      downloads7d: 0,
      mime: f.mime
    }));

    render();
  }catch(err){
    toast("danger", "Error", err.message);
  }
}

function getVisibleFiles(){
  let list = [...state.files];
  if (state.search.trim()){
    const q = state.search.trim().toLowerCase();
    list = list.filter(f => f.name.toLowerCase().includes(q));
  }
  if (state.filter !== "all"){
    list = list.filter(f => f.visibility === state.filter);
  }
  switch(state.sort){
    case "oldest": list.sort((a,b)=> new Date(a.updatedAt) - new Date(b.updatedAt)); break;
    case "newest": list.sort((a,b)=> new Date(b.updatedAt) - new Date(a.updatedAt)); break;
    case "name_asc": list.sort((a,b)=> a.name.localeCompare(b.name)); break;
    case "name_desc": list.sort((a,b)=> b.name.localeCompare(a.name)); break;
    case "size_asc": list.sort((a,b)=> a.size - b.size); break;
    case "size_desc": list.sort((a,b)=> b.size - a.size); break;
  }
  return list;
}

function render(){
  const list = getVisibleFiles();
  els.tbody.innerHTML = "";

  if (state.files.length === 0){
    els.empty.hidden = false;
    $("#filesTable").hidden = true;
  } else {
    $("#filesTable").hidden = false;
    els.empty.hidden = list.length !== 0;
  }

  for (const f of list){
    const tr = document.createElement("tr");
    tr.innerHTML = `
      <td><input type="checkbox" class="chkRow" data-id="${f.id}"></td>
      <td>
        <div class="file">
          <div class="file__icon">${iconFor(f.ext)}</div>
          <div>
            <div class="file__name">${escapeHtml(f.name)}</div>
            <div class="file__sub">${escapeHtml(f.originalName)}</div>
          </div>
        </div>
      </td>
      <td>${escapeHtml(f.ext.toUpperCase())}</td>
      <td>${formatBytes(f.size)}</td>
      <td>
        <span class="badge ${f.visibility === "public" ? "badge--public" : "badge--private"}">
          ${f.visibility === "public" ? "🌐 Público" : "🔒 Privado"}
        </span>
      </td>
      <td>${formatDate(f.updatedAt)}</td>
      <td>
        <div class="actions">
          <button class="btn btn--ghost" data-action="download" data-id="${f.id}">⬇ Descargar</button>
          <button class="btn btn--ghost" data-action="share" data-id="${f.id}">🔗 Compartir</button>
          <button class="btn btn--ghost" data-action="toggle" data-id="${f.id}">
            ${f.visibility === "public" ? "🔒 Privatizar" : "🌐 Publicar"}
          </button>
          <button class="btn btn--danger" data-action="delete" data-id="${f.id}">🗑 Eliminar</button>
        </div>
      </td>
    `;
    els.tbody.appendChild(tr);
  }

  const total = state.files.length;
  const totalBytes = state.files.reduce((a,f)=> a + (f.size || 0), 0);
  const publics = state.files.filter(f=> f.visibility === "public").length;
  const downloads = state.files.reduce((a,f)=> a + (f.downloads7d || 0), 0);

  els.statFiles.textContent = total;
  els.statStorage.textContent = formatBytes(totalBytes);
  els.statPublic.textContent = publics;
  els.statDownloads.textContent = downloads;

  els.chkAll.checked = false;
}

function openModal(){
  els.modal.classList.add("is-open");
  els.modal.setAttribute("aria-hidden","false");
}
function closeModal(){
  els.modal.classList.remove("is-open");
  els.modal.setAttribute("aria-hidden","true");
  els.uploadForm.reset();
  els.filePicked.hidden = true;
}
function setPickedFile(file){
  if (!file){ els.filePicked.hidden = true; return; }
  els.filePicked.hidden = false;
  els.pickedName.textContent = file.name;
  els.pickedMeta.textContent = `${formatBytes(file.size)} · ${file.type || "tipo desconocido"}`;
}

/* Upload real */
async function uploadReal(file){
  // El backend guarda original_name y lo sube como private por default.
  // (Luego si quieres, hacemos que respete el select visibility)
  const uploaded = await apiUpload(file);

  // Si el usuario seleccionó “public” en UI, lo aplicamos con PATCH
  const desired = els.visibilitySelect?.value || "private";
  if (desired === "public"){
    await apiJson(`/api/files/${uploaded.id}/visibility`, "PATCH", { visibility: "public" });
  }

  toast("success","Subido",`${uploaded.original_name} agregado.`);
  closeModal();
  await loadFiles();
}

/* Table actions real */
async function handleTableClick(e){
  const btn = e.target.closest("button[data-action]");
  if (!btn) return;

  const id = btn.dataset.id;
  const action = btn.dataset.action;
  const file = state.files.find(f=> f.id === id);
  if (!file) return;

  try{
    if (action === "download"){
      // descarga real (abre en otra pestaña)
      window.open(`${API_BASE}/api/files/${id}/download`, "_blank");
      toast("success","Descarga",`Descargando: ${file.name}`);
      return;
    }

    if (action === "share"){
      // por ahora link demo (cuando creemos endpoint público lo volvemos real)
      const link = `${API_BASE}/api/files/${id}/download`;
      await navigator.clipboard?.writeText(link);
      toast("info","Enlace copiado","Se copió enlace de descarga (requiere sesión).");
      return;
    }

    if (action === "toggle"){
      const newVis = file.visibility === "public" ? "private" : "public";
      await apiJson(`/api/files/${id}/visibility`, "PATCH", { visibility: newVis });
      toast("success","Visibilidad",`${file.name} ahora es ${newVis}.`);
      await loadFiles();
      return;
    }

    if (action === "delete"){
      await apiJson(`/api/files/${id}`, "DELETE");
      toast("danger","Eliminado",`${file.name} eliminado.`);
      await loadFiles();
      return;
    }
  }catch(err){
    toast("danger","Error", err.message);
  }
}

async function bulkDelete(){
  const selected = $$(".chkRow").filter(c=> c.checked).map(c=> c.dataset.id);
  if (selected.length === 0){
    toast("info","Sin selección","Selecciona archivos.");
    return;
  }

  try{
    for (const id of selected){
      await apiJson(`/api/files/${id}`, "DELETE");
    }
    toast("danger","Eliminación masiva",`Se eliminaron ${selected.length} archivos.`);
    await loadFiles();
  }catch(err){
    toast("danger","Error", err.message);
  }
}

function init(){
  // show UI
  els.appView.hidden = false;

  // user
  const u = session.user || { name:"Usuario", role:"cliente" };
  els.userName.textContent = u.name || "Usuario";
  els.userRole.textContent = u.role || "—";
  els.userAvatar.textContent = avatarFromName(u.name || "U");

  // sidebar mobile
  els.btnSidebar.addEventListener("click", ()=> els.sidebar.classList.toggle("is-open"));

  // nav
  $$(".nav__item").forEach(a=>{
    a.addEventListener("click",(e)=>{
      e.preventDefault();
      $$(".nav__item").forEach(x=> x.classList.remove("nav__item--active"));
      a.classList.add("nav__item--active");
      $("#pageTitle").textContent = a.textContent.trim();
      $("#pageSubtitle").textContent = "Archivos reales (Rust API + MongoDB).";
      if (window.innerWidth <= 900) els.sidebar.classList.remove("is-open");
    });
  });

  // filters / sort / search
  $$(".segmented__btn").forEach(btn=>{
    btn.addEventListener("click", ()=>{
      $$(".segmented__btn").forEach(b=> b.classList.remove("is-active"));
      btn.classList.add("is-active");
      state.filter = btn.dataset.filter;
      render();
    });
  });
  els.sortSelect.addEventListener("change", ()=> { state.sort = els.sortSelect.value; render(); });
  els.searchInput.addEventListener("input", ()=> { state.search = els.searchInput.value; render(); });
  els.btnRefresh.addEventListener("click", ()=> loadFiles());

  // table actions
  els.tbody.addEventListener("click", handleTableClick);

  // select all
  els.chkAll.addEventListener("change", ()=> $$(".chkRow").forEach(c=> c.checked = els.chkAll.checked));

  // bulk delete
  els.btnBulkDelete.addEventListener("click", bulkDelete);

  // modal upload
  els.btnOpenUpload.addEventListener("click", openModal);
  els.btnEmptyUpload.addEventListener("click", openModal);
  els.modal.addEventListener("click", (e)=> { if (e.target.closest("[data-close='1']")) closeModal(); });
  window.addEventListener("keydown", (e)=> { if (e.key === "Escape" && els.modal.classList.contains("is-open")) closeModal(); });

  els.dropzone.addEventListener("drop", (e)=>{
    e.preventDefault();
    const file = e.dataTransfer.files?.[0];
    if (file){ els.fileInput.files = e.dataTransfer.files; setPickedFile(file); }
  });
  els.dropzone.addEventListener("dragover", (e)=> e.preventDefault());

  els.fileInput.addEventListener("change", ()=> setPickedFile(els.fileInput.files?.[0]));

  els.uploadForm.addEventListener("submit", async (e)=>{
    e.preventDefault();
    const file = els.fileInput.files?.[0];
    if (!file){ toast("danger","Falta archivo","Selecciona un archivo."); return; }
    await uploadReal(file);
  });

  // logout
  els.btnLogout.addEventListener("click", ()=>{
    clearSession();
    toast("info","Sesión cerrada","Redirigiendo...");
    setTimeout(()=> location.href = "./login.html", 350);
  });

  // load real files
  loadFiles();
}

init();
