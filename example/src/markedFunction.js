export const markedFunction = (str) => {
  return str.toUpperCase();
}

export function anotherMarkedFunction(str) {
  return str.toLowerCase();
}

window.markedFunction = markedFunction;
window.anotherMarkedFunction = anotherMarkedFunction;
