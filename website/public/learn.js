// Turns a Learn track page into a step-by-step guide. Each level-2 heading after
// the [data-learn-track] marker starts a step. Without JavaScript the page reads
// top to bottom; with it, one step shows at a time with progress kept per browser.
(() => {
  const marker = document.querySelector('[data-learn-track]');
  const content = marker?.closest('.sl-markdown-content');
  if (!content) return;
  const track = marker.dataset.learnTrack;
  const storageKey = `tsrust-learn:${track}`;
  const isStep = (node) => node.matches?.('h2, .sl-heading-wrapper.level-h2');
  const starts = [...content.children].filter((node) => isStep(node) && (marker.compareDocumentPosition(node) & Node.DOCUMENT_POSITION_FOLLOWING));
  if (starts.length < 2) return;

  const steps = starts.map((start) => {
    const heading = start.matches('h2') ? start : start.querySelector('h2');
    const section = document.createElement('section');
    section.className = 'learn-step';
    section.setAttribute('aria-labelledby', heading.id);
    start.before(section);
    let node = start;
    while (node && !(node !== start && isStep(node))) {
      const next = node.nextElementSibling;
      section.append(node);
      node = next;
    }
    return { id: heading.id, title: heading.textContent.trim(), heading, section };
  });

  const read = () => {
    try { return JSON.parse(localStorage.getItem(storageKey)) ?? {}; } catch { return {}; }
  };
  const write = () => {
    try { localStorage.setItem(storageKey, JSON.stringify({ current, done: [...done], all: showAll })); } catch { /* storage unavailable */ }
  };
  const saved = read();
  const done = new Set(Array.isArray(saved.done) ? saved.done.filter((id) => steps.some((step) => step.id === id)) : []);
  let current = Math.max(0, steps.findIndex((step) => step.id === saved.current));
  let showAll = saved.all === true;

  const nav = document.createElement('nav');
  nav.className = 'learn-nav not-content';
  nav.setAttribute('aria-label', 'Track steps');
  const progress = document.createElement('p');
  progress.className = 'learn-progress';
  progress.setAttribute('aria-live', 'polite');
  const list = document.createElement('ol');
  const items = steps.map((step, index) => {
    const item = document.createElement('li');
    const link = document.createElement('a');
    link.href = `#${step.id}`;
    link.textContent = step.title;
    link.addEventListener('click', (event) => { event.preventDefault(); go(index, true); });
    item.append(link);
    list.append(item);
    return { item, link };
  });
  const toggle = document.createElement('button');
  toggle.type = 'button';
  toggle.className = 'learn-toggle';
  toggle.addEventListener('click', () => { showAll = !showAll; render(); write(); });
  nav.append(progress, list, toggle);
  marker.append(nav);

  for (const [index, step] of steps.entries()) {
    const controls = document.createElement('div');
    controls.className = 'learn-controls not-content';
    const button = (label, action, className) => {
      const element = document.createElement('button');
      element.type = 'button';
      element.textContent = label;
      element.className = className;
      element.addEventListener('click', action);
      controls.append(element);
      return element;
    };
    if (index > 0) button('← Previous', () => go(index - 1, true), 'learn-previous');
    step.complete = button('', () => {
      if (done.has(step.id)) done.delete(step.id); else done.add(step.id);
      if (done.has(step.id) && index + 1 < steps.length && !showAll) go(index + 1, true); else render();
      write();
    }, 'learn-complete');
    if (index + 1 < steps.length) button('Next →', () => go(index + 1, true), 'learn-next');
    step.section.append(controls);
  }

  function render() {
    progress.textContent = `${done.size} of ${steps.length} steps complete`;
    toggle.textContent = showAll ? 'Show one step at a time' : 'Show all steps';
    for (const [index, step] of steps.entries()) {
      step.section.hidden = !showAll && index !== current;
      const finished = done.has(step.id);
      items[index].item.classList.toggle('learn-done', finished);
      if (index === current) items[index].link.setAttribute('aria-current', 'step'); else items[index].link.removeAttribute('aria-current');
      step.complete.textContent = finished ? 'Completed ✓' : (index + 1 < steps.length ? 'Mark complete and continue' : 'Mark complete');
      step.complete.setAttribute('aria-pressed', String(finished));
    }
  }

  function go(index, focus) {
    current = index;
    render();
    write();
    const step = steps[index];
    if (focus) {
      history.replaceState(null, '', `#${step.id}`);
      step.heading.setAttribute('tabindex', '-1');
      step.heading.focus({ preventScroll: true });
      step.heading.scrollIntoView({ block: 'start' });
    }
  }

  // Table-of-contents links and shared URLs select the step holding the target.
  function fromHash() {
    const id = decodeURIComponent(location.hash.slice(1));
    const target = id && document.getElementById(id);
    const index = target ? steps.findIndex((step) => step.section.contains(target)) : -1;
    if (index === -1) return false;
    current = index;
    render();
    write();
    target.scrollIntoView({ block: 'start' });
    return true;
  }
  window.addEventListener('hashchange', fromHash);
  if (!fromHash()) render();
})();
