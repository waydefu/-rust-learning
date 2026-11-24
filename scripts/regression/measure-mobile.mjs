// Real local HTTP; Chromium emulation, not an Android hardware measurement.
import { chromium, devices } from '@playwright/test';
import { mkdir, writeFile } from 'node:fs/promises';

const base=process.env.TEST_ORIGIN || 'http://127.0.0.1:4173/-rust-learning/';
const origin=new URL(base);
if (!['127.0.0.1','localhost'].includes(origin.hostname)) throw new Error('Measure a local preview, not the production site');
const browser=await chromium.launch({headless:true});
const samples=[];
try {
  for(const path of ['', 'chapters/ownership/']){
    for(let sample=1;sample<=3;sample++){
      const context=await browser.newContext({...devices['Pixel 7'], reducedMotion:'no-preference'});
      try {
        const page=await context.newPage();
        const errors=[];
        page.on('pageerror', error=>errors.push(error.message));
        const cdp=await context.newCDPSession(page);
        await cdp.send('Network.enable');
        await cdp.send('Network.setCacheDisabled',{cacheDisabled:true});
        await cdp.send('Network.emulateNetworkConditions',{
          offline:false, latency:150, downloadThroughput:200000, uploadThroughput:93750,
          connectionType:'cellular3g',
        });
        await cdp.send('Emulation.setCPUThrottlingRate',{rate:4});
        await page.addInitScript(()=>{
          const values={lcp:null,cls:0};
          window.__rustPerf=values;
          if(PerformanceObserver.supportedEntryTypes.includes('largest-contentful-paint')){
            new PerformanceObserver(list=>{
              for(const e of list.getEntries()) values.lcp=e.startTime;
            }).observe({type:'largest-contentful-paint',buffered:true});
          }
          if(PerformanceObserver.supportedEntryTypes.includes('layout-shift')){
            let total=0, first=0, last=0;
            new PerformanceObserver(list=>{
              for(const e of list.getEntries()){
                if(e.hadRecentInput)continue;
                if(!first||e.startTime-last>1000||e.startTime-first>5000){first=e.startTime;total=0;}
                total+=e.value;last=e.startTime;values.cls=Math.max(values.cls,total);
              }
            }).observe({type:'layout-shift',buffered:true});
          }
        });
        const response=await page.goto(new URL(path,origin).href,{waitUntil:'load',timeout:45000});
        if(!response?.ok())throw new Error('HTTP navigation failed '+path);
        await page.evaluate(()=>document.fonts.ready);
        await page.waitForTimeout(1500);
        const metrics=await page.evaluate(()=>{
          const nav=performance.getEntriesByType('navigation')[0];
          const resources=performance.getEntriesByType('resource');
          const paint=performance.getEntriesByName('first-contentful-paint')[0];
          return {
            fcpMs:paint?.startTime ?? null,
            lcpMs:window.__rustPerf.lcp,
            cls:window.__rustPerf.cls,
            domContentLoadedMs:nav.domContentLoadedEventEnd,
            loadMs:nav.loadEventEnd,
            transferredBytes:nav.transferSize+resources.reduce((sum,r)=>sum+r.transferSize,0),
            resources:resources.length,
            overflow:document.documentElement.scrollWidth>innerWidth+1,
          };
        });
        if(errors.length||metrics.overflow||metrics.fcpMs===null||metrics.lcpMs===null)throw new Error('Mobile measurement invalid '+JSON.stringify({path,errors,metrics}));
        samples.push({path: path||'/',sample,...metrics});
      }finally{await context.close();}
    }
  }
}finally{await browser.close();}
const median=(items,key)=>items.map(x=>x[key]).sort((a,b)=>a-b)[Math.floor(items.length/2)];
const summary=['/','chapters/ownership/'].map(path=>{
  const group=samples.filter(x=>x.path===path);
  return {path,...Object.fromEntries(['fcpMs','lcpMs','cls','domContentLoadedMs','loadMs','transferredBytes','resources'].map(key=>[key,median(group,key)]))};
});
await mkdir('.quality-evidence',{recursive:true});
const report={method:'Chromium Pixel 7 simulation, real local HTTP, cold cache, 4x CPU, 150ms latency, 1.6Mbps down / 0.75Mbps up, three runs per route',samples,summary};
await writeFile('.quality-evidence/mobile-performance.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report,null,2));
