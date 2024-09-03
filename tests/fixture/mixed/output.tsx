// @ts-nocheck
/*---BEGIN MARK_EXPRESSIONS–--
[["import",[{"shouldMark":true,"webpackChunkName":"ImportTrue"}],["./importTrue"],"input.js:19:19"],["import",[{"shouldMark":"True","webpackChunkName":"ImportTrueStr"}],["./importTrueStr"],"input.js:20:22"],["import",[{"shouldMark":1,"webpackChunkName":"Import1"}],["./import1"],"input.js:21:20"],["import",[{"shouldMark":{},"webpackChunkName":"ImportObj"}],["./importObj"],"input.js:22:22"],["import",[{"shouldMark":[],"webpackChunkName":"ImportArr"}],["./importArr"],"input.js:23:22"],["markFnA",["markFnA",true],"input.js:37:4"],["markFnB",["markFnB",false],"input.js:38:4"],["markFnC",["markFnC",{"0":"a","a":0,"b":null,"c":null}],"input.js:39:4"],["this","markThisFnA",["this.markThisFnA",0,1,2,"4","5","6"],"input.js:48:4"],["this","markThisFnB",["this.markThisFnB",0,1,2,"4","5","6"],"input.js:49:4"],["this","markThisFnC",["this.markThisFnC",0,1,2,"4","5","6"],"input.js:50:4"],["markFnA",["propA"],"input.js:57:13"],["this","markThisFnB",["propThisA"],"input.js:58:38"],["window","markWindowFnA",["propWindowA"],"input.js:59:25"],["markFnA",["childA",0,true,[],null,{"component":null,"test":true}],"input.js:62:9"]]
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
const importTrue = import(/* webpackChunkName: "ImportTrue", shouldMark: true */ "./importTrue");
const importTrueStr = import(/* webpackChunkName: "ImportTrueStr", shouldMark: "True" */ "./importTrueStr");
const import1 = ()=>import(/* webpackChunkName: "Import1", shouldMark: 1 */ "./import1");
const importObj = ()=>import(/* webpackChunkName: "ImportObj", shouldMark: {} */ "./importObj");
const importArr = ()=>import(/* webpackChunkName: "ImportArr", shouldMark: [] */ "./importArr");
const importEmptyStr = import(/* webpackChunkName: "ImportEmptyStr", shouldMark: "" */ "./importEmptyStr");
const import0 = import(/* webpackChunkName: "Import0", shouldMark: 0 */ "./import0");
const import0f = import(/* webpackChunkName: "Import0.0", shouldMark: 0.0 */ "./import0f");
const importFalse = import(/* webpackChunkName: "ImportFalse", shouldMark: false */ "./importFalse");
const object = {
    markFnA,
    markFnB,
    markFnC,
    markThisFnA: markFnA,
    markThisFnB: markFnB,
    markThisFnC: markFnC,
    markedFn: (...args)=>{
        markFnA("markFnA", true);
        markFnB("markFnB", false);
        markFnC("markFnC", {
            0: "a",
            "a": 0,
            "b": window,
            "c": null
        });
        this.markFnA("should not work");
        this.markFnB("should not work");
        this.markFnC("should not work");
        window.markFnA("should not work", "0", "1", "2", ...args);
        window.markFnB("should not work", "0", "1", "2", ...args);
        window.markFnC("should not work", "0", "1", "2", ...args);
        this.markThisFnA("this.markThisFnA", 0, 1, 2, "4", "5", "6");
        this.markThisFnB("this.markThisFnB", 0, 1, 2, "4", "5", "6");
        this.markThisFnC("this.markThisFnC", 0, 1, 2, "4", "5", "6");
    }
};
const Component = ()=>{
    return <div propA={markFnA("propA")} propThisA={function() {
        return this.markThisFnB("propThisA");
    }} propWindowA={()=>window.markWindowFnA("propWindowA")}>
      <div>
        {markFnA("childA", 0, true, [], Component, {
        test: true,
        component: Component
    })}
        {markThisFnA("should not work")}
        {markWindowFnA("should not work")}
        {this.markFnA("should not work")}
        {window.markFnA("should not work")}
      </div>
    </div>;
};
