const batchSize = 10;
const page = document.querySelector(".search-page");
const input = page.querySelector(".search-input");
const status = page.querySelector(".search-status");
const results = page.querySelector(".search-results");
const more = page.querySelector(".search-more");
const clearButton = page.querySelector(".search-clear");
const segmenter = new Intl.Segmenter(document.documentElement.lang || undefined, {
  granularity: "word",
});

let engine;
let matches = [];
let shown = 0;
let request = 0;
let debounce;

const words = text =>
  [...segmenter.segment(text)]
    .filter(part => part.isWordLike)
    .map(part => part.segment.toLocaleLowerCase());

const loadEngine = () =>
  (engine ??= Promise.all([
    import("./minisearch.js"),
    fetch(new URL("./search-index.json", import.meta.url)).then(response => {
      if (!response.ok) throw new Error(`Search index returned ${response.status}`);
      return response.json();
    }),
  ]).then(([{ default: MiniSearch }, documents]) => {
    const index = new MiniSearch({
      fields: ["title", "categories", "content"],
      processTerm: words,
      searchOptions: {
        prefix: true,
        fuzzy: 0.2,
        combineWith: "AND",
        boost: { title: 3, categories: 2 },
      },
    });
    index.addAll(documents);

    return { index, documents };
  }));

const transpositions = term => {
  if (!/^[a-z0-9]{4,}$/i.test(term)) return [term];

  const variants = new Set([term]);
  for (let index = 0; index < term.length - 1; index++) {
    variants.add(
      term.slice(0, index)
      + term[index + 1]
      + term[index]
      + term.slice(index + 2),
    );
  }

  return [...variants];
};

const searchQuery = query => ({
  combineWith: "AND",
  queries: words(query).map(term => ({
    combineWith: "OR",
    queries: transpositions(term),
  })),
});

const excerpt = (entry, match) => {
  const terms = Object.keys(match).sort((left, right) => right.length - left.length);
  const source = entry.content;
  if (!source) return;

  const normalized = source.toLocaleLowerCase();
  const positions = terms
    .map(term => normalized.indexOf(term.toLocaleLowerCase()))
    .filter(position => position >= 0);
  const position = positions.length ? Math.min(...positions) : 0;
  let start = Math.max(0, position - 60);
  const end = Math.min(source.length, start + 180);
  start = Math.max(0, end - 180);

  return {
    text: `${start ? "…" : ""}${source.slice(start, end).trim()}${end < source.length ? "…" : ""}`,
    terms,
  };
};

const highlight = (element, text, terms) => {
  const pattern = terms
    .filter(Boolean)
    .map(term => term.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"))
    .join("|");
  if (!pattern) {
    element.textContent = text;
    return;
  }

  const matcher = new RegExp(pattern, "giu");
  let offset = 0;
  for (const match of text.matchAll(matcher)) {
    element.append(document.createTextNode(text.slice(offset, match.index)));
    const mark = document.createElement("mark");
    mark.textContent = match[0];
    element.append(mark);
    offset = match.index + match[0].length;
  }
  element.append(document.createTextNode(text.slice(offset)));
};

const resultItem = ({ entry, result }) => {
  const item = document.createElement("li");
  item.className = "search-result";

  const title = document.createElement("h2");
  const link = document.createElement("a");
  link.href = entry.url;
  const terms = Object.keys(result.match);
  if (entry.breadcrumbs) {
    const breadcrumbs = document.createElement("span");
    breadcrumbs.className = "search-result-breadcrumbs";
    highlight(breadcrumbs, entry.breadcrumbs, terms);
    link.append(breadcrumbs, document.createTextNode(" / "));
  }
  const label = document.createElement("span");
  highlight(label, entry.title, terms);
  link.append(label);
  title.append(link);
  item.append(title);

  const metadata = [entry.published, entry.categories].filter(Boolean);
  if (metadata.length) {
    const meta = document.createElement("p");
    meta.className = "search-result-meta";
    meta.textContent = metadata.join(" · ");
    item.append(meta);
  }

  const summary = excerpt(entry, result.match);
  if (summary) {
    const paragraph = document.createElement("p");
    paragraph.className = "search-result-summary";
    highlight(paragraph, summary.text, summary.terms);
    item.append(paragraph);
  }

  return item;
};

const showMore = current => {
  if (current !== request) return;

  const batch = matches.slice(shown, shown + batchSize);
  results.append(...batch.map(resultItem));
  shown += batch.length;
  more.hidden = shown >= matches.length;
};

const search = async () => {
  const current = ++request;
  const query = input.value.trim();
  const isCurrent = () => current === request && input.value.trim() === query;
  const url = new URL(location.href);
  query ? url.searchParams.set("q", query) : url.searchParams.delete("q");
  history.replaceState(null, "", url);
  more.hidden = true;

  if (!query) {
    page.classList.remove("searching");
    matches = [];
    shown = 0;
    results.replaceChildren();
    status.textContent = "";
    return;
  }

  page.classList.add("searching");
  try {
    const { index, documents } = await loadEngine();
    if (!isCurrent()) return;

    let found = index.search(query);
    const typoQuery = searchQuery(query);
    if (!found.length && typoQuery.queries.length) found = index.search(typoQuery);
    matches = found.map(result => ({ result, entry: documents[result.id] }));
    shown = 0;
    results.replaceChildren();
    showMore(current);

    const count = matches.length;
    status.textContent = count === 1 ? "1 result" : `${count} results`;
  } catch {
    if (isCurrent()) status.textContent = "Search is unavailable.";
  } finally {
    if (isCurrent()) page.classList.remove("searching");
  }
};

const scheduleSearch = () => {
  clearTimeout(debounce);
  const query = input.value.trim();
  clearButton.hidden = !input.value;
  page.classList.toggle("searching", Boolean(query));
  if (!query) {
    void search();
    return;
  }
  debounce = setTimeout(() => void search(), 300);
};

input.value = new URL(location.href).searchParams.get("q") ?? "";
clearButton.hidden = !input.value;
page.querySelector("form").addEventListener("submit", event => {
  event.preventDefault();
  clearTimeout(debounce);
  void search();
});
clearButton.addEventListener("click", () => {
  input.value = "";
  input.focus();
  scheduleSearch();
});
input.addEventListener("input", scheduleSearch);
more.addEventListener("click", () => showMore(request));
void search();
