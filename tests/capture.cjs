const { chromium } = require('playwright');
const fs = require('fs');
async function main() {
 const dir = process.env.OUTPUT_DIR || '/artifacts/screenshots'; fs.mkdirSync(dir,{recursive:true});
 const browser = await chromium.launch({headless:true,args:['--no-sandbox']});
 const page = await browser.newPage({viewport:{width:1440,height:1000},locale:'ru-RU'});
 const failures=[]; page.on('pageerror', error=>failures.push(error.message));
 const base=process.env.WEB_URL || 'http://nginx';
 for(const [name,url] of [['dashboard','/dashboard'],['iss','/iss'],['osdr','/osdr'],['cms','/page/welcome']]){
  const response=await page.goto(base+url,{waitUntil:'networkidle',timeout:90000});
  if(response.status()!==200)throw new Error(`${url}: ${response.status()}`);
  if(name==='dashboard'){
   await page.locator('#jwstFilter').waitFor();
   await page.waitForFunction(()=>document.querySelector('#jwstInfo').textContent.includes('Показано'),null,{timeout:15000});
   const captions=page.locator('#jwstTrack figure');
   if(await captions.count()!==1)throw new Error('Gallery fixture did not render');
   if(await page.locator('#astroBody tr').count()<1)throw new Error('Events did not render');
  }
  const bodyBounds=await page.locator('body').boundingBox();
  await page.setViewportSize({width:1440,height:Math.max(300,Math.ceil(bodyBounds.height)+24)});
  await page.waitForTimeout(200);
  await page.screenshot({path:`${dir}/${name}.png`,fullPage:true});
 }
 await browser.close();
 if(failures.length)throw new Error(JSON.stringify(failures));
 console.log(JSON.stringify({event:'browser_checks_passed',pages:4,consoleErrors:0}));
}
main().catch(e=>{console.error(e);process.exit(1)});
