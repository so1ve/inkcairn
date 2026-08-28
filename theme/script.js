document.getElementById("theme-toggle")?.addEventListener("click", () => {
  const root = document.documentElement;
  root.dataset.theme = root.dataset.theme === "dark" ? "light" : "dark";

  try {
    localStorage.setItem("inkcairn:theme", root.dataset.theme);
  } catch {}
});

for (const button of document.querySelectorAll(".copy-code")) {
  button.addEventListener("click", event => copyCode(button, event.detail > 0));
}

async function copyCode(button, releaseFocus) {
  const code = button.parentElement.querySelectorAll(".code-line:not(.diff-del)");
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

const lightbox = document.getElementById("image-lightbox");
const lightboxImage = lightbox.querySelector("img");
let sourceImage;

for (const image of document.querySelectorAll(".markdown-body p img")) {
  image.tabIndex = 0;
  image.setAttribute("role", "button");
  image.setAttribute("aria-haspopup", "dialog");

  image.addEventListener("click", event => {
    event.preventDefault();
    openImage(image);
  });

  image.addEventListener("keydown", event => {
    if (event.key !== "Enter" && event.key !== " ") return;

    event.preventDefault();
    openImage(image);
  });
}

lightbox.addEventListener("click", event => {
  if (event.target === lightbox || event.target === lightboxImage) closeImage();
});

lightbox.addEventListener("cancel", event => {
  event.preventDefault();
  closeImage();
});

function openImage(image) {
  sourceImage = image;
  lightboxImage.src = image.currentSrc;
  lightboxImage.alt = image.alt;
  image.classList.add("image-zoom-transition");

  const show = () => {
    image.classList.remove("image-zoom-transition");
    lightboxImage.classList.add("image-zoom-transition");
    lightbox.showModal();
  };

  if (document.startViewTransition) document.startViewTransition(show);
  else show();
}

function closeImage() {
  const hide = () => {
    lightbox.close();
    lightboxImage.classList.remove("image-zoom-transition");
    sourceImage.classList.add("image-zoom-transition");
  };

  const transition = document.startViewTransition?.(hide);
  if (!transition) hide();

  (transition?.finished ?? Promise.resolve()).finally(() => {
    sourceImage.classList.remove("image-zoom-transition");
    sourceImage = undefined;
  });
}
