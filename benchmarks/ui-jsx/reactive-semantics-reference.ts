// Independent transition model; no compiler/runtime source is imported.
function state(changed:boolean){return {label:changed?'Renamed draft':'Draft',description:changed?'metadata-error':'metadata-hint',invalid:changed?'true':null,hidden:changed,toggle:changed?'Restore metadata':'Change metadata',badge:changed?'Updated status':'Initial status',image:changed?'Saved':null,role:changed?'img':'presentation'};}
console.log(JSON.stringify({steps:[state(false),state(true),state(false),state(true)]}));
