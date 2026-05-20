import "./style.css";

const app = document.querySelector<HTMLDivElement>("#app");

if (!app) {
  throw new Error("missing #app root");
}

app.innerHTML = `
  <main class="shell">
    <header class="hero">
      <p class="eyebrow">Phase 0 scaffold</p>
      <h1>Reversi replay visualizer</h1>
      <p class="summary">
        Phaser will own the board canvas. Lightweight controls and artifact
        loading stay in the surrounding shell.
      </p>
    </header>
  </main>
`;
