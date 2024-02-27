var func = function() {
  return window.markedFunction('window');
}

(function() {
  window.property = window['markedFunction']('property');
})()

