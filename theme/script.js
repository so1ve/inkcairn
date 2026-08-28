document.getElementById("theme-toggle")?.addEventListener("click", () => {
  const root = document.documentElement;
  root.dataset.theme = root.dataset.theme === "dark" ? "light" : "dark";

  try {
    localStorage.setItem("inkcairn:theme", root.dataset.theme);
  } catch {}
});

document.addEventListener("click", event => {
  const button = event.target.closest(".copy-code");
  if (button) copyCode(button, event.detail > 0);
});

async function copyCode(button, releaseFocus) {
  const code = button.parentElement.querySelectorAll(".code-text:not(.diff-del)");
  const text = [...code].map(line => line.textContent).join("\n");

  try {
    await navigator.clipboard.writeText(text);
  } catch {
    return;
  }

  button.dataset.copied = "";
  button.ariaLabel = "Copied";
  if (releaseFocus) button.blur();

  setTimeout(() => {
    delete button.dataset.copied;
    button.ariaLabel = "Copy code";
  }, 1200);
}
