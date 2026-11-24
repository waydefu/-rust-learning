for(const [name,operation,base] of [
  ['root-build-check','build','/'],
  ['pages-build-check','build','/-rust-learning/'],
  ['invalid-base-preserves-output','invalid','/../'],
  ['missing-source-preserves-output','missing-source','/'],
  ['rust-examples','examples','/'],
  ['pages-preview-http','preview','/-rust-learning/']
])console.log(JSON.stringify({name,args:[operation,base]}));
