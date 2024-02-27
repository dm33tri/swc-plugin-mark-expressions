function x() {
    function y() {
        const z = ()=>{
            console.log(format`${markedFunction(/* markExpression: nested */ 'nested')}`);
        };
    }
    (function() {
        markedFunction(/* markExpression: ternary */ 'ternary') ? y() : y(markedFunction(/* markExpression: another */ 'another'));
    })();
}
