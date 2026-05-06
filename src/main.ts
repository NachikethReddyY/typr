import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface Settings {
  microphone: string;
  engine: string; // "auto" | "local" | "cloud"
  whisperModel: string;
  groqApiKey: string;
  cloudProvider: string; // "groq" | "mistral"
  mistralApiKey: string;
  recordingMode: string;
  hotkey: string;
}

interface MicDevice {
  name: string;
  is_default: boolean;
}

interface DownloadProgress {
  downloaded: number;
  total: number;
  percent: number;
}

// DOM elements
const statusDot = document.getElementById("status-dot")!;
const statusText = document.getElementById("status-text")!;
const micSelect = document.getElementById("mic-select") as HTMLSelectElement;
const engineAuto = document.getElementById("engine-auto")!;
const engineLocal = document.getElementById("engine-local")!;
const engineCloud = document.getElementById("engine-cloud")!;
const autoHint = document.getElementById("auto-hint")!;
const localSettings = document.getElementById("local-settings")!;
const cloudSettings = document.getElementById("cloud-settings")!;
const groqSettings = document.getElementById("groq-settings")!;
const mistralSettings = document.getElementById("mistral-settings")!;
const modelSelect = document.getElementById("model-select") as HTMLSelectElement;
const downloadBtn = document.getElementById("download-btn")!;
const downloadProgress = document.getElementById("download-progress")!;
const progressFill = document.getElementById("progress-fill")!;
const groqKey = document.getElementById("groq-key") as HTMLInputElement;
const mistralKey = document.getElementById("mistral-key") as HTMLInputElement;
const cloudProviderRadios = document.querySelectorAll('input[name="cloud-provider"]') as NodeListOf<HTMLInputElement>;
const modeToggle = document.getElementById("mode-toggle")!;
const modePtt = document.getElementById("mode-ptt")!;
const hotkeyText = document.getElementById("hotkey-text")!;

// Section navigation
const navItems = document.querySelectorAll(".nav-item");
const sections = document.querySelectorAll(".content-section");

navItems.forEach((item) => {
  item.addEventListener("click", () => {
    const target = item.getAttribute("data-section");
    navItems.forEach((n) => n.classList.remove("active"));
    sections.forEach((s) => s.classList.remove("active"));
    item.classList.add("active");
    document.getElementById(`section-${target}`)?.classList.add("active");
  });
});

// Window drag — titlebar and sidebar empty space
const titlebar = document.getElementById("titlebar")!;
const sidebar = document.getElementById("sidebar")!;
const appWindow = getCurrentWindow();

titlebar.addEventListener("mousedown", (e) => {
  if ((e.target as HTMLElement).closest("button, select, input, a, .nav-item")) return;
  appWindow.startDragging();
});

sidebar.addEventListener("mousedown", (e) => {
  if ((e.target as HTMLElement).closest("button, select, input, a, .nav-item")) return;
  appWindow.startDragging();
});

let currentSettings: Settings;

async function loadSettings() {
  currentSettings = await invoke<Settings>("get_settings");

  // Populate mic dropdown
  const mics = await invoke<MicDevice[]>("list_microphones");
  micSelect.innerHTML = "";
  mics.forEach((mic) => {
    const option = document.createElement("option");
    option.value = mic.name;
    option.textContent = mic.name + (mic.is_default ? " (default)" : "");
    micSelect.appendChild(option);
  });
  micSelect.value = currentSettings.microphone;

  // Engine
  setEngine(currentSettings.engine);

  // Model
  modelSelect.value = currentSettings.whisperModel;
  await checkModelStatus();

  // Cloud provider
  cloudProviderRadios.forEach(radio => {
    radio.checked = radio.value === currentSettings.cloudProvider;
  });
  updateCloudSettingsVisibility();

  // API keys
  groqKey.value = currentSettings.groqApiKey;
  mistralKey.value = currentSettings.mistralApiKey;

  // Recording mode
  setRecordingMode(currentSettings.recordingMode);

  // Hotkey
  hotkeyText.textContent = currentSettings.hotkey.replace("CmdOrCtrl", "Cmd");
}

function setEngine(engine: string) {
  currentSettings.engine = engine;
  engineAuto.classList.toggle("active", engine === "auto");
  engineLocal.classList.toggle("active", engine === "local");
  engineCloud.classList.toggle("active", engine === "cloud");
  autoHint.classList.toggle("hidden", engine !== "auto");
  localSettings.classList.toggle("hidden", engine !== "local" && engine !== "auto");
  cloudSettings.classList.toggle("hidden", engine !== "cloud");
  updateLocalSettingsVisibility();
}

function updateLocalSettingsVisibility() {
  const showLocal = currentSettings.engine === "local" || 
    (currentSettings.engine === "auto");
  localSettings.classList.toggle("hidden", !showLocal);
}

function updateCloudSettingsVisibility() {
  const provider = currentSettings.cloudProvider;
  groqSettings.classList.toggle("hidden", provider !== "groq");
  mistralSettings.classList.toggle("hidden", provider !== "mistral");
}

function setRecordingMode(mode: string) {
  currentSettings.recordingMode = mode;
  modeToggle.classList.toggle("active", mode === "toggle");
  modePtt.classList.toggle("active", mode === "push-to-talk");
}

async function checkModelStatus() {
  const downloaded = await invoke<boolean>("check_model_downloaded", {
    modelSize: modelSelect.value,
  });
  downloadBtn.textContent = downloaded ? "\u2713" : "Download";
  (downloadBtn as HTMLButtonElement).disabled = downloaded;
}

async function saveSettings() {
  currentSettings.microphone = micSelect.value;
  currentSettings.whisperModel = modelSelect.value;
  currentSettings.groqApiKey = groqKey.value;
  currentSettings.cloudProvider = Array.from(cloudProviderRadios).find(r => r.checked)?.value || "groq";
  currentSettings.mistralApiKey = mistralKey.value;
  await invoke("save_settings", { settings: currentSettings });
}

// Event listeners
engineAuto.addEventListener("click", () => {
  setEngine("auto");
  saveSettings();
});

engineLocal.addEventListener("click", () => {
  setEngine("local");
  saveSettings();
});

engineCloud.addEventListener("click", () => {
  setEngine("cloud");
  saveSettings();
});

cloudProviderRadios.forEach(radio => {
  radio.addEventListener("change", () => {
    currentSettings.cloudProvider = radio.value;
    updateCloudSettingsVisibility();
    saveSettings();
  });
});

mistralKey.addEventListener("change", () => saveSettings());

micSelect.addEventListener("change", () => saveSettings());

modelSelect.addEventListener("change", async () => {
  await checkModelStatus();
  saveSettings();
});

downloadBtn.addEventListener("click", async () => {
  (downloadBtn as HTMLButtonElement).disabled = true;
  downloadProgress.classList.remove("hidden");
  progressFill.style.width = "0%";

  try {
    await invoke("download_model", { modelSize: modelSelect.value });
    downloadBtn.textContent = "\u2713";
  } catch (e) {
    downloadBtn.textContent = "Retry";
    (downloadBtn as HTMLButtonElement).disabled = false;
    console.error("Download failed:", e);
  }
  downloadProgress.classList.add("hidden");
});

groqKey.addEventListener("change", () => saveSettings());

modeToggle.addEventListener("click", () => {
  setRecordingMode("toggle");
  saveSettings();
});

modePtt.addEventListener("click", () => {
  setRecordingMode("push-to-talk");
  saveSettings();
});

// Listen for recording state changes
listen<string>("recording-state", (event) => {
  const state = event.payload;
  statusDot.className = "";
  if (state === "Recording") {
    statusDot.classList.add("recording");
    statusText.textContent = "Recording...";
  } else if (state === "Transcribing") {
    statusDot.classList.add("transcribing");
    statusText.textContent = "Transcribing...";
  } else {
    statusDot.classList.add("ready");
    statusText.textContent = "Ready";
  }
});

// Listen for download progress
listen<DownloadProgress>("download-progress", (event) => {
  const { percent } = event.payload;
  progressFill.style.width = `${percent}%`;
});

// Initialize
loadSettings();
