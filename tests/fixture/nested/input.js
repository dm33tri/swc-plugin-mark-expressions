function x() {
  function y() {
    const z = () => {
      console.log(format`${markedFunction('nested')}`)
    }
  }

  (function () {
    markedFunction('ternary') ? y() : y(markedFunction('another'));
  })();
}
