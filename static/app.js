const $ = id => document.getElementById(id);
let history = [], previous = null, activeView = 'overview';
let blocksLoaded = 0, mempoolLoaded = 0;
const cadence = 15000, detailCadence = 60000;
const fmt = n => n == null ? '—' : new Intl.NumberFormat('en-US').format(n);
const bytes = n => n == null ? '—' : n < 1024 ? `${fmt(n)} B` : n < 1048576 ? `${(n / 1024).toFixed(1)} KB` : `${(n / 1048576).toFixed(1)} MB`;
const satRate = n => n == null || !Number.isFinite(n) || n <= 0 ? 'Unavailable' : n < 0.005 ? '<0.01' : n.toFixed(2);
function set(id, value) { $(id).textContent = value; }
function stamp(value) { return new Date(Number(value) * 1000).toLocaleTimeString(); }
function btc(value) { return value == null ? 'Unavailable' : `${value.toFixed(4)} BTC`; }
function duration(s) { if (s == null) return '—'; const d=Math.floor(s/86400),h=Math.floor(s%86400/3600); return d?`${d}d ${h}h`:`${h}h ${Math.floor(s%3600/60)}m`; }
function age(time) { if(time==null)return 'Time unavailable';const sec=Math.max(0,Math.floor(Date.now()/1000-time));return sec<60?`${sec}s ago`:sec<3600?`${Math.floor(sec/60)}m ago`:sec<86400?`${Math.floor(sec/3600)}h ago`:`${Math.floor(sec/86400)}d ago`; }
function diagnosticText(category) {
  return ({unavailable_rpc:'Bitcoin Core RPC is unavailable',timeout:'Bitcoin Core RPC timed out',unsupported_method:'This Bitcoin Core version does not support the requested data',pruned_data:'Required block data is unavailable because of pruning',malformed_result:'Bitcoin Core returned data in an unexpected format',active_chain_mismatch:'The block changed during an active-chain check',output_limit:'The response exceeded the dashboard size limit'})[category] || 'Data is temporarily unavailable';
}
function draw() { const c=$('chart'),r=c.getBoundingClientRect(),dpr=devicePixelRatio||1;c.width=r.width*dpr;c.height=r.height*dpr;const x=c.getContext('2d');x.scale(dpr,dpr);const max=Math.max(1,...history.flatMap(v=>[v.down,v.up]));for(const [key,color] of [['down','#55d6a2'],['up','#f7931a']]){x.beginPath();x.strokeStyle=color;x.lineWidth=2;history.forEach((v,i)=>{const px=i*(r.width/Math.max(1,history.length-1)),py=r.height-(v[key]/max)*(r.height-8)-4;i?x.lineTo(px,py):x.moveTo(px,py)});x.stroke();} }
async function json(url) { const r=await fetch(url,{cache:'no-store'}); if(!r.ok) throw Error('unavailable'); return r.json(); }
function renderFees(container, fees=[]) {
  container.replaceChildren();
  for (const f of fees) {
    const item=document.createElement('div'),label=document.createElement('small'),value=document.createElement('strong');
    item.className='fee';label.textContent=`${fmt(f.target)} blocks`;value.textContent=satRate(f.sat_per_vbyte);
    item.append(label,value);
    const unit=document.createElement('small');unit.textContent='sat/vB';item.append(unit);container.append(item);
  }
  if (!fees.length) { const empty=document.createElement('span');empty.className='muted';empty.textContent='Fee estimates unavailable';container.append(empty); }
}
async function refreshOverview() {
  const s=await json('/api/status'),n=s.node,h=s.host;
  set('state',n.reachable?'Bitcoin Core is reachable':'Bitcoin Core is unavailable');$('state').className=n.reachable?'':'offline';
  set('meta',n.reachable?`Updated ${stamp(s.updated_at)} · ${n.ibd?'Initial block download in progress':n.background_validation===true?'Background validation active':'Node responding'}`:'The dashboard is running. It will retry automatically.');
  set('version',n.version||'Bitcoin Core');set('chain',n.chain||'Chain unavailable');set('height',`Block height ${fmt(n.blocks)}`);set('headers',`Headers ${fmt(n.headers)}`);
  const p=n.progress==null?null:Math.max(0,Math.min(100,n.progress*100));set('progress',p==null?'Unavailable':`${p.toFixed(2)}%`);$('bar').style.width=`${p||0}%`;
  set('peers',n.inbound==null?'Unavailable':fmt(n.inbound+n.outbound));set('peer-detail',n.inbound==null?'Peer counts unavailable':`${fmt(n.inbound)} inbound · ${fmt(n.outbound)} outbound`);
  set('mempool',fmt(n.mempool_count));set('mem-detail',`${bytes(n.mempool_vsize)} vsize · ${bytes(n.mempool_usage)} memory · ${bytes(n.mempool_max)} capacity`);
  set('validation',n.ibd==null?'Unavailable':n.ibd?'In progress':'Complete');set('cpu',h.cpu_percent==null?'Unavailable':`${h.cpu_percent.toFixed(1)}%`);set('memory',h.memory_used==null?'Unavailable':`${bytes(h.memory_used)} / ${bytes(h.memory_total)}`);set('disk',bytes(h.disk_free));set('uptime',duration(h.uptime));
  const down=previous&&n.download!=null?Math.max(0,n.download-previous.download)*1000/cadence:0,up=previous&&n.upload!=null?Math.max(0,n.upload-previous.upload)*1000/cadence:0;
  if(n.download!=null&&n.upload!=null){set('download',`${bytes(down)}/s`);set('upload',`${bytes(up)}/s`);history.push({down,up});history=history.slice(-36);draw();previous={download:n.download,upload:n.upload};}
  renderFees($('fee-list'),n.fees);set('updated',`Last checked ${stamp(s.updated_at)}`);
}
function renderBands(bands) {
  const list=$('fee-bands');list.replaceChildren();
  if(!bands||!bands.length){const note=document.createElement('p');note.className='muted';note.textContent='Fee bands are unavailable because no template transactions were returned.';list.append(note);return;}
  const max=Math.max(1,...bands.map(b=>b.virtual_size||0));let hasData=false;
  bands.forEach((band,index)=>{
    if(!band.transaction_count)return;hasData=true;
    const row=document.createElement('div'),label=document.createElement('span'),bar=document.createElement('span'),fill=document.createElement('span'),stats=document.createElement('small');
    row.className='fee-band-row';label.className='band-label';label.textContent=`${band.label} sat/vB`;bar.className='band-bar';fill.className=`band-fill band-${index}`;fill.style.width=`${Math.max(1,Math.min(100,(band.virtual_size/max)*100))}%`;bar.append(fill);stats.className='band-stats';stats.textContent=`${fmt(band.transaction_count)} tx · ${bytes(band.virtual_size)}`;row.append(label,bar,stats);list.append(row);
  });
  if(!hasData){const note=document.createElement('p');note.className='muted';note.textContent='The template returned no transactions with usable fee and weight fields.';list.append(note);}
}
async function refreshMempool() {
  const m=await json('/api/mempool');set('mempool-notice',m.local_node_notice);
  set('mempool-state',m.status?diagnosticText(m.status):'Local snapshot available');
  set('mp-count',fmt(m.transaction_count));set('mp-vsize',bytes(m.virtual_size));set('mp-memory',bytes(m.memory_usage));set('mp-capacity',`Capacity ${bytes(m.max_memory)}`);
  set('mp-min-fee',m.minimum_fee_sat_per_vbyte==null?'Unavailable':`${satRate(m.minimum_fee_sat_per_vbyte)} sat/vB`);
  set('mp-min-relay',m.minimum_relay_fee_sat_per_vbyte==null?'Unavailable':`${satRate(m.minimum_relay_fee_sat_per_vbyte)} sat/vB`);
  renderFees($('mp-fees'),m.fees);renderBands(m.fee_bands);
  if(m.template){const t=m.template;$('template-metrics').hidden=false;set('template-height',fmt(t.height));set('template-count',fmt(t.transaction_count));set('template-vsize',bytes(t.virtual_size));set('template-fees',btc(t.total_fees_btc));set('template-status','Current candidate assembled by this node. It is not mined and does not predict which transactions a mining pool will select.');}
  else {$('template-metrics').hidden=true;set('template-status',`Local template unavailable: ${diagnosticText(m.template_status)}`);}
  set('mempool-updated',`Last updated ${stamp(m.updated_at)} · refreshes every 60 sec`);mempoolLoaded=Date.now();
}
function renderBlocks(data, containerId, statusId, limit) {
  const list=$(containerId);list.replaceChildren();
  for(const b of (data.blocks||[]).slice(0,limit)){
    const card=document.createElement('button');card.className='block-card';card.type='button';card.setAttribute('aria-label',`Confirmed block ${fmt(b.height)}, ${fmt(b.tx_count)} transactions`);
    const top=document.createElement('span'),height=document.createElement('strong'),ageLabel=document.createElement('small'),hash=document.createElement('small'),stats=document.createElement('span'),fees=document.createElement('small');
    top.className='block-top';height.textContent=`#${fmt(b.height)}`;ageLabel.textContent=age(b.time);top.append(height,ageLabel);hash.className='block-hash';hash.textContent=b.hash?`${b.hash.slice(0,12)}…`:'';
    stats.className='block-stats';stats.textContent=`${fmt(b.tx_count)} tx · ${bytes(b.tx_size)} tx data · ${fmt(b.tx_weight)} WU`;
    fees.textContent=b.total_fees_btc==null?'Fee summary unavailable':`${b.total_fees_btc.toFixed(4)} BTC fees`;
    card.append(top,hash,stats,fees);card.addEventListener('click',()=>loadBlockDetail(b.height));list.append(card);
  }
  if(!data.blocks?.length){const empty=document.createElement('p');empty.className='muted';empty.textContent=data.diagnostic?`Recent blocks unavailable: ${diagnosticText(data.diagnostic)}.`:'No recent blocks are available from the current best chain.';list.append(empty);}
  else if(data.skipped_blocks){set(statusId,`Showing ${data.blocks.length} verified block(s); ${data.skipped_blocks} card(s) unavailable (${diagnosticText(data.diagnostic)}).`);return;}
  set(statusId,`Tip ${fmt(data.tip_height)} · ${data.blocks.length} verified block(s) · checked ${stamp(data.updated_at)}`);
}
async function loadBlocks(containerId='block-list', limit=6, statusId='blocks-updated') {
  const data=await json('/api/blocks');renderBlocks(data,containerId,statusId,limit);blocksLoaded=Date.now();
}
async function loadBlockDetail(height) {
  const panel=$('block-detail'),content=$('detail-content');panel.hidden=false;content.textContent='Loading block details…';
  try{const d=await json(`/api/blocks/${encodeURIComponent(height)}`),b=d.block;content.replaceChildren();const title=document.createElement('h3'),summary=document.createElement('p'),list=document.createElement('ol');title.textContent=`Confirmed block #${fmt(b.height)}`;summary.textContent=`${b.hash} · ${age(b.time)} · ${fmt(b.confirmations)} confirmations · ${fmt(b.tx_count)} transactions · ${b.total_fees_btc==null?'fees unavailable':`${b.total_fees_btc.toFixed(4)} BTC fees`}`;for(const txid of d.transactions){const li=document.createElement('li');li.textContent=txid;list.append(li);}content.append(title,summary,list);if(d.transactions_truncated){const note=document.createElement('p');note.className='muted';note.textContent='Transaction list capped at 100 IDs.';content.append(note);}}
  catch{content.textContent='Block details are temporarily unavailable. The block may have changed during a chain reorganization; refresh the list to verify.';}
}
function reportLoadError(id){set(id,'Dashboard data is temporarily unavailable. Retry shortly.');}
function selectView(name) {
  activeView=name;
  document.querySelectorAll('[role="tab"]').forEach(tab=>{const selected=tab.id===`tab-${name}`;tab.setAttribute('aria-selected',String(selected));tab.tabIndex=selected?0:-1;});
  document.querySelectorAll('[role="tabpanel"]').forEach(panel=>panel.hidden=panel.id!==`view-${name}`);
  if(name==='blocks'&&Date.now()-blocksLoaded>=detailCadence)loadBlocks().catch(()=>reportLoadError('blocks-updated'));
  if(name==='mempool'){
    if(Date.now()-mempoolLoaded>=detailCadence)refreshMempool().catch(()=>reportLoadError('mempool-state'));
    if(Date.now()-blocksLoaded>=detailCadence)loadBlocks('mempool-blocks',3,'mempool-block-status').catch(()=>reportLoadError('mempool-block-status'));
  }
}
document.querySelectorAll('[role="tab"]').forEach((tab,index,tabs)=>{tab.addEventListener('click',()=>selectView(tab.id.slice(4)));tab.addEventListener('keydown',event=>{let next=index;if(event.key==='ArrowRight')next=(index+1)%tabs.length;else if(event.key==='ArrowLeft')next=(index+tabs.length-1)%tabs.length;else if(event.key==='Home')next=0;else if(event.key==='End')next=tabs.length-1;else return;event.preventDefault();tabs[next].focus();selectView(tabs[next].id.slice(4));});});
$('open-blocks').addEventListener('click',()=>{selectView('blocks');$('tab-blocks').focus();});
$('close-detail').addEventListener('click',()=>{$('block-detail').hidden=true;});
$('refresh').addEventListener('click',async()=>{const btn=$('refresh');btn.disabled=true;try{await refreshOverview();if(activeView==='blocks')await loadBlocks();if(activeView==='mempool'){await Promise.all([refreshMempool(),loadBlocks('mempool-blocks',3,'mempool-block-status')]);}}catch{set('state','Dashboard status unavailable');$('state').className='offline';set('meta','Check that the local dashboard is running, then retry.');}finally{btn.disabled=false;}});
const refreshTimer=setInterval(()=>{refreshOverview().catch(()=>{set('state','Dashboard status unavailable');$('state').className='offline';});if(activeView==='blocks'&&Date.now()-blocksLoaded>=detailCadence)loadBlocks().catch(()=>reportLoadError('blocks-updated'));if(activeView==='mempool'){if(Date.now()-mempoolLoaded>=detailCadence)refreshMempool().catch(()=>reportLoadError('mempool-state'));if(Date.now()-blocksLoaded>=detailCadence)loadBlocks('mempool-blocks',3,'mempool-block-status').catch(()=>reportLoadError('mempool-block-status'));}},cadence);
refreshOverview().catch(()=>{});
const shutdownButton=$('shutdown');shutdownButton.addEventListener('click',async()=>{if(!window.confirm('Stop the Bitcoin Node Dashboard web server? Bitcoin Core, Tor, and Bisq will keep running.'))return;shutdownButton.disabled=true;try{const token=document.querySelector('meta[name="dashboard-shutdown-token"]').content;const response=await fetch('/api/shutdown',{method:'POST',headers:{'X-Dashboard-Shutdown-Token':token},cache:'no-store'});if(!response.ok)throw Error();clearInterval(refreshTimer);set('state','Bitcoin Node Dashboard has stopped');$('state').className='';set('meta','The dashboard server has stopped. Close this page.');$('refresh').disabled=true;shutdownButton.textContent='Dashboard stopped';}catch{shutdownButton.disabled=false;set('meta','Shutdown was not accepted. The dashboard is still running.');}});
