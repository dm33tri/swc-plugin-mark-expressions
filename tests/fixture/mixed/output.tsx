// @ts-nocheck
/*---BEGIN MARK_EXPRESSIONS–-- 
[
    ["markThisFnA", "this", ["this.markThisFnA", null]],
    ["markThisFnB", "this", ["this.markThisFnB", null]],
    ["markThisFnC", "this", ["this.markThisFnC", null]],
    ["import", { webpackChunkName: "ImportA", shouldMark: true }, ["./importA"]],
    ["import", { webpackChunkName: "ImportB", shouldMark: 1 }, ["./importB"]],
    ["import", { webpackChunkName: "ImportC", shouldMark: false }, ["./importC"]],
    ["markFnA", ["ImportC", null]],
    ["markFnA", ["markFnA"]],
    ["markWindowFnB", "window", ["window.markWindowFnB", null]],
    ["markFnA", ["propA={markFnA()}"]],
    ["markThisFnB", "this", ["propThisA={this.markThisFnB()}"]],
    ["markWindowFnA", "window", ["propWindowA={window.markWindowFnA()}"]]
]
 ---END MARK_EXPRESSIONS---*/ const markFnA = (...args)=>{
    console.log("markFnA", ...args);
};
const markFnB = (...args)=>{
    console.log("markFnB", ...args);
};
const markFnC = (...args)=>{
    console.log("markFnC", ...args);
};
window.markWindowFnA = markFnA;
window.markWindowFnB = markFnB;
window.markWindowFnC = markFnC;
const object = {
    markFnA,
    markFnB,
    markFnC,
    markThisFnA: markFnA,
    markThisFnB: markFnB,
    markThisFnC: markFnC,
    markedFn: (...args)=>{
        this.markThisFnA("this.markThisFnA", ...args);
        this.markThisFnB("this.markThisFnB", ...args);
        this.markThisFnC("this.markThisFnC", ...args);
        this.markFnA("should not work", ...args);
        this.markFnB("should not work", ...args);
        this.markFnC("should not work", ...args);
    }
};
const importA = import(/* webpackChunkName: "ImportA", shouldMark: true */ "./importA");
const importB = ()=>import(/* webpackChunkName: "ImportB", shouldMark: 1 */ "./importB");
const importC = markFnA("ImportC", ()=>import(/* webpackChunkName: "ImportC", shouldMark: false */ "./importC"));
const Component = ()=>{
    const Wrapper = (...args)=><Wrapper>

      {import(/* webpackChunkName: "ImportD" */ "./importD")}

      {markFnA("markFnA")}

      {window.markWindowFnB("window.markWindowFnB", ...args)}

    </Wrapper>;
    return <Wrapper propA={markFnA("propA={markFnA()}")} propThisA={function() {
        return this.markThisFnB("propThisA={this.markThisFnB()}");
    }} propWindowA={window.markWindowFnA("propWindowA={window.markWindowFnA()}")}>

      <div>

        {markThisFnA("should not work")}

        {markWindowFnA("should not work")}

        {this.markFnA("should not work")}

        {window.markFnA("should not work")}

      </div>

    </Wrapper>;
};
