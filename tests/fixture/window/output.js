var func = (function() {
    return window.markedFunction(/* markExpression: window */ 'window');
})(function() {
    window.property = window['markedFunction'](/* markExpression: property */ 'property');
})();
