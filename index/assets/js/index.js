function scrollToTop(duration) {
  const start = window.scrollY;
  const startTime = 'now' in window.performance ? performance.now() : new Date().getTime();
  const easeInOutQuad = (t) => t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t;
  function scroll() {
    const now = 'now' in window.performance ? performance.now() : new Date().getTime();
    const time = Math.min(1, ((now - startTime) / duration));
    const timeFunction = easeInOutQuad(time);
    window.scrollTo(0, Math.ceil(timeFunction * (0 - start) + start));
    if (window.scrollY !== 0) {
      requestAnimationFrame(scroll);
    }
  }
  requestAnimationFrame(scroll);
}
window.onload = function() {
  document.getElementById("sidebar").onclick = function() {
    scrollToTop(800);
  }
};