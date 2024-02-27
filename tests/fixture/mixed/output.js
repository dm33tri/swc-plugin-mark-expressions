import { markedFunction } from './markedFunction.js';
export function literal() {
    return {
        value: markedFunction(/* markExpression: literal */ 'literal')
    };
}
export function variable(value) {
    return {
        value: markedFunction(value)
    };
}
export const multiple = (count, key)=>{
    const substitute = markedFunction(/* markExpression: substitute */ 'substitute', count);
    const substituteLit = markedFunction(/* markExpression: substitute */ 'substitute', 'lit');
    const numeric = markedFunction(0);
    const dynamicSubstiture = markedFunction(key, count);
};
const empty = markedFunction();
export default {
    value: markedFunction(/* markExpression: default */ 'default')
};
