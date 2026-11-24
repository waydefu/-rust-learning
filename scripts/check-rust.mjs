import { catalog, readLesson } from '../src/course.mjs';
import { applyNativeFences, verifyNativeExamples } from '../src/native-render.mjs';

const lessons=await Promise.all(catalog.map(readLesson));
applyNativeFences(lessons);
const {version,compiled,run,error,project,messages}=verifyNativeExamples(lessons);
console.log(version);
for(const message of messages) console.log(message);
console.log(`Rust 區塊通過 ${compiled+error+project} 段（獨立編譯 ${compiled} 段，其中 ${run} 段執行或執行測試，預期診斷 ${error} 段，Cargo 專案 ${project} 段）`);
