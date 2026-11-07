// A sigil spec for every live unit (mission arena#34, slice B): each unit's
// portrait as its designer would draw it, by unit name. `body` is the silhouette
// (bold, flat, outlined, readable at 44 px, one of a kind), `mark` is the sign of
// its rule, and `aw` grows something on the awoken form (the 3rd copy). The
// numbers are the sigil grammar's expressions (mobile/ui/sigil/grammar.ts):
// y runs down, one unit is the portrait circle's radius.
//
// Every spec passes the sigil `validate`; portraits.test.ts walks all of them,
// `npm run mvp:portraits` draws them and checks that no two share a silhouette.
// The first 16 are the UI duel's hand-written specs (bodies as drawn there, a
// mark layer added where `aw` grew nothing); a new unit's comes from
// its LLM (server/src/mvp/portrait.ts) and rides on the idea, not in this file.
import type { Spec } from "../../mobile/ui/sigil/spec.js";

export const PORTRAITS: Record<string, Spec> = {
  // ---- the pool's units, in pool order
  // Hourglass: from the UI duel
  Hourglass: {
    body: [
      {"shape":"rect","count":2,"w":0.78,"h":0.1,"rad":0.03,"y":"pick(i,-0.6,0.6)","fill":"ember"},
      {"shape":"poly","n":3,"r":0.42,"y":-0.24,"rot":3.1416,"fill":"paper","alpha":0.9},
      {"shape":"poly","n":3,"r":0.42,"y":0.24,"fill":"paper","alpha":0.9},
      {"shape":"poly","n":3,"r":"0.3*(1-fr(t/3))","y":-0.2,"rot":3.1416,"fill":"gold","ol":0},
      {"shape":"poly","n":3,"r":"0.3*fr(t/3)","y":"0.42-0.15*fr(t/3)","fill":"gold","ol":0},
      {"shape":"circle","count":2,"r":0.045,"y":"0.02+0.36*fr(t*1.2+i*0.37)","fill":"gold","ol":0},
    ],
    mark: [
      {"shape":"circle","count":3,"r":0.07,"x":0.66,"y":"-0.22+0.22*i","fill":"gold","alpha":"0.3+0.7*step(charge,i+1)","glow":"step(charge,i+1)"},
      {"shape":"circle","count":3,"r":0.05,"x":"-0.14+0.14*i","y":"0.5-0.12*sin(t*3+i*2)","fill":"gold","alpha":"aw","ol":0},
    ],
  },
  // Mirror
  Mirror: {
    body: [
      {"shape":"ellipse","rx":0.62,"ry":0.86,"fill":"gold"},
      {"shape":"ellipse","rx":0.46,"ry":0.7,"fill":{"grad":["frost","arcane"]}},
    ],
    mark: [
      {"shape":"rect","count":"2+aw","w":0.07,"h":0.5,"rad":0.03,"x":"-0.2+0.2*i+0.25*sin(t*1.2)","y":-0.1,"rot":0.55,"fill":"paper","alpha":"0.75-0.15*i","ol":0},
    ],
  },
  // Candle: from the UI duel
  Candle: {
    body: [
      {"shape":"ellipse","rx":0.56,"ry":0.14,"y":0.66,"fill":"gold"},
      {"shape":"rect","w":0.52,"h":"0.22+0.56*hp","rad":0.06,"y":"0.62-(0.22+0.56*hp)/2","fill":{"grad":["paper","bone"]}},
      {"shape":"petal","len":"0.4+0.05*sin(t*9)","w":0.2,"y":"0.56-(0.22+0.56*hp)","rot":"0.08*sin(t*5)","fill":{"grad":["paper","ember"]},"glow":0.7},
    ],
    mark: [
      {"shape":"circle","r":"0.3+0.04*sin(t*9)","y":"0.36-(0.22+0.56*hp)","fill":"gold","alpha":0.22,"ol":0,"blend":"add"},
      {"shape":"petal","count":2,"len":0.24,"w":0.1,"x":"pick(i,-0.3,0.3)","y":"0.4-(0.22+0.56*hp)","rot":"pick(i,-0.6,0.6)","fill":"ember","alpha":"aw","ol":0},
    ],
  },
  // Gambler: from the UI duel
  Gambler: {
    body: [
      {"shape":"group","x":-0.2,"y":0.12,"rot":"-0.25+0.12*sin(t*2)","layers":[{"shape":"rect","w":0.6,"h":0.6,"rad":0.12,"fill":"paper"},{"shape":"circle","count":5,"r":0.06,"x":"pick(i,-0.16,0.16,0,-0.16,0.16)","y":"pick(i,-0.16,-0.16,0,0.16,0.16)","fill":"ink","ol":0}]},
      {"shape":"group","x":0.3,"y":-0.22,"rot":"0.45-0.15*sin(t*2+1)","layers":[{"shape":"rect","w":0.46,"h":0.46,"rad":0.1,"fill":"owner"},{"shape":"circle","count":2,"r":0.055,"x":"pick(i,-0.1,0.1)","y":"pick(i,-0.1,0.1)","fill":"ink","ol":0}]},
    ],
    mark: [
      {"shape":"ellipse","rx":"0.17*abs(cos(t*3))+0.02","ry":0.17,"x":0.42,"y":0.42,"fill":"gold"},
      {"shape":"star","n":4,"r":"0.16*fire","r2":"0.05*fire","x":0.42,"y":0.42,"fill":"paper","ol":0,"blend":"add"},
      {"shape":"rect","w":0.3,"h":0.3,"rad":0.07,"x":-0.58,"y":-0.5,"rot":"0.3*sin(t*2)","fill":"gold","alpha":"aw"},
    ],
  },
  // Nautilus: from the UI duel
  Nautilus: {
    body: [
      {"shape":"circle","count":9,"r":"(0.055+0.034*i)*(1+0.05*sin(t*2-i*0.6))","x":"(0.04+0.058*i)*cos(i*0.8+0.3)","y":"(0.04+0.058*i)*sin(i*0.8+0.3)","fill":{"grad":["owner.light","owner"]}},
    ],
    mark: [
      {"shape":"ring","r":0.84,"w":"0.03+0.035*min(shield,3)","stroke":"frost","alpha":"0.35+0.2*min(shield,3)","ol":0},
      {"shape":"circle","r":0.2,"x":0.62,"y":0.12,"fill":"owner.light","alpha":"aw"},
    ],
  },
  // Magnet
  Magnet: {
    body: [
      {"shape":"arc","r":0.5,"a":4.6,"rot":3.14,"y":-0.02,"w":0.3,"stroke":"blood"},
      {"shape":"rect","count":2,"w":0.3,"h":0.22,"x":"pick(i,-0.38,0.38)","y":-0.45,"fill":"paper"},
    ],
    mark: [
      {"shape":"circle","count":"3+aw","r":0.07,"x":"pick(i,-0.8,0.1,0.75,-0.4)*(1-0.3*fr(t*0.7+i*0.33))","y":"-0.8+0.45*fr(t*0.7+i*0.33)","fill":"venom","alpha":"1-fr(t*0.7+i*0.33)","ol":0},
    ],
  },
  // Raven: from the UI duel
  Raven: {
    body: [
      {"shape":"petal","len":0.78,"w":0.3,"x":-0.05,"y":0.22,"rot":"-2.3+0.12*sin(t*3)","fill":"owner.dark"},
      {"shape":"circle","r":0.44,"x":0.05,"y":-0.05,"fill":{"grad":["owner","ink"]}},
      {"shape":"poly","n":3,"r":0.2,"x":0.52,"y":-0.04,"rot":1.571,"fill":"gold"},
      {"shape":"circle","r":0.075,"x":0.2,"y":-0.15,"fill":"paper","ol":0},
    ],
    mark: [
      {"shape":"rect","w":0.24,"h":0.3,"rad":0.04,"x":-0.48,"y":"-0.5+0.04*sin(t*2)","rot":"0.25*sin(t)","fill":"paper","alpha":"0.55+0.45*fire"},
      {"shape":"petal","count":3,"len":0.3,"w":0.1,"x":"-0.1+0.2*i","y":-0.42,"rot":"(i-1)*0.45","fill":"owner.dark","alpha":"aw"},
    ],
  },
  // Seed
  Seed: {
    body: [
      {"shape":"ellipse","rx":0.6,"ry":0.13,"y":0.76,"fill":"owner.dark"},
      {"shape":"ellipse","rx":0.42,"ry":0.5,"y":0.38,"fill":"bone"},
      {"shape":"line","len":"0.4+0.25*aw","w":0.1,"y":0.2,"stroke":"venom"},
      {"shape":"petal","count":2,"len":"0.45+0.25*aw+0.03*sin(t*2)","w":0.24,"x":"pick(i,-0.02,0.02)","y":"0.0-0.15*aw","rot":"pick(i,-1.05,1.05)+0.06*sin(t*2+i)","fill":"venom"},
    ],
    mark: [
      {"shape":"circle","r":0.14,"x":0.62,"y":-0.62,"fill":"paper","alpha":"1-aw"},
      {"shape":"circle","r":0.12,"x":0.69,"y":-0.66,"fill":"night","alpha":"1-aw","ol":0},
    ],
  },
  // Sniper
  Sniper: {
    body: [
      {"shape":"group","count":4,"rot":"i*1.571","layers":[{"shape":"line","len":0.45,"w":0.11,"y":-0.5,"stroke":"paper"}]},
      {"shape":"ring","r":0.55,"w":0.24,"stroke":"night"},
      {"shape":"ring","r":0.55,"w":0.05,"stroke":"paper","ol":0},
      {"shape":"circle","r":"0.1+0.03*beat","fill":"blood"},
    ],
    mark: [
      {"shape":"circle","r":0.1,"x":"0.28*sin(t*1.1)","y":"0.2*cos(t*1.7)","fill":"blood","glow":"0.5+fire"},
      {"shape":"ring","count":"1+aw","r":"0.22+0.12*i","w":0.05,"stroke":"blood","ol":0},
    ],
  },
  // Medic: from the UI duel
  Medic: {
    body: [
      {"shape":"circle","r":0.6,"s":"1+0.05*beat","fill":{"radial":["paper","bone"]}},
      {"shape":"rect","w":0.6,"h":0.2,"rad":0.04,"s":"1+0.05*beat","fill":"blood"},
      {"shape":"rect","w":0.2,"h":0.6,"rad":0.04,"s":"1+0.05*beat","fill":"blood"},
    ],
    mark: [
      {"shape":"circle","count":4,"r":0.065,"x":"0.8*cos(t*0.8+i*1.571)","y":"0.8*sin(t*0.8+i*1.571)","fill":"venom","glow":"fire"},
      {"shape":"ring","r":0.68,"w":0.05,"stroke":"venom","alpha":"aw*(0.5+0.5*beat)","ol":0},
    ],
  },
  // Leech
  Leech: {
    body: [
      {"shape":"ellipse","rx":0.74,"ry":0.34,"x":0.1,"y":0.18,"rot":"-0.5+0.05*sin(t*3)","fill":{"grad":["blood","night"]}},
      {"shape":"circle","r":0.28,"x":-0.55,"y":"-0.02+0.03*sin(t*3)","fill":"blood"},
      {"shape":"poly","count":2,"n":3,"r":0.1,"x":"pick(i,-0.64,-0.46)","y":0.2,"rot":3.14,"fill":"paper"},
      {"shape":"circle","r":0.05,"x":-0.55,"y":-0.1,"fill":"paper","ol":0},
    ],
    mark: [
      {"shape":"petal","count":"1+aw","len":"0.28+0.1*fire","w":0.12,"x":"0.5+0.17*i","y":"-0.3-0.3*fire","fill":"venom"},
    ],
  },
  // Lone Wolf
  "Lone Wolf": {
    body: [
      {"shape":"poly","count":2,"n":3,"r":0.34,"x":"pick(i,-0.38,0.38)","y":-0.5,"rot":"pick(i,-0.35,0.35)+0.14*beat*pick(i,1,-1)","fill":"bone"},
      {"shape":"ellipse","rx":0.58,"ry":0.5,"y":0.05,"fill":{"grad":["paper","bone"]}},
      {"shape":"ellipse","rx":0.24,"ry":0.2,"y":0.3,"fill":"paper"},
      {"shape":"circle","r":0.08,"y":0.24,"fill":"ink","ol":0},
      {"shape":"circle","count":2,"r":0.07,"x":"pick(i,-0.22,0.22)","y":-0.05,"fill":"gold","ol":0},
    ],
    mark: [
      {"shape":"star","n":4,"r":"0.12+0.14*aw","r2":0.04,"x":-0.68,"y":-0.62,"fill":"gold","ol":0,"alpha":"0.5+0.5*beat"},
      {"shape":"star","n":4,"r":"0.12+0.14*aw","r2":0.04,"x":0.68,"y":-0.62,"fill":"gold","ol":0,"alpha":"aw*(0.5+0.5*beat)"},
    ],
  },
  // Drummer: from the UI duel
  Drummer: {
    body: [
      {"shape":"rect","w":0.96,"h":0.5,"rad":0.08,"y":0.2,"fill":{"grad":["owner","owner.dark"]}},
      {"shape":"line","count":3,"len":0.42,"w":0.05,"x":"-0.3+0.3*i","y":0.43,"rot":"0.5*(1-2*fr(i/2))","stroke":"gold","ol":0},
      {"shape":"ellipse","rx":0.48,"ry":0.15,"y":"-0.05+0.03*beat","fill":"paper"},
      {"shape":"line","count":2,"len":0.62,"w":0.08,"x":"pick(i,-0.12,0.12)","y":"-0.06","rot":"pick(i,-0.75,0.75)*(1-0.35*beat)","stroke":"bone"},
    ],
    mark: [
      {"shape":"arc","count":"2+aw","r":"0.48+0.13*i+0.05*beat","a":1.3,"w":0.05,"y":0.05,"stroke":"paper","alpha":"0.8-0.25*i","ol":0},
      {"shape":"circle","count":3,"r":0.08,"x":"-0.22+0.22*i","y":0.66,"fill":"gold","alpha":"0.3+0.7*step(charge,i+1)","glow":"step(charge,i+1)"},
    ],
  },
  // Bodyguard: from the UI duel
  Bodyguard: {
    body: [
      {"shape":"poly","n":6,"r":"0.72+0.025*sin(t*2)","rot":0.52,"x":"-0.06*hit","fill":{"grad":["owner","owner.dark"]}},
      {"shape":"poly","n":6,"r":0.5,"rot":0.52,"x":"-0.06*hit","fill":"owner.dark","ol":0.03},
      {"shape":"rect","w":0.56,"h":0.13,"rad":0.05,"y":-0.08,"x":"-0.06*hit","fill":"ink"},
      {"shape":"circle","count":2,"r":"0.045*step(fr(t/3),0.06)","x":"pick(i,-0.13,0.13)-0.06*hit+0.05*sin(t*1.3)","y":-0.08,"fill":"gold","ol":0,"glow":0.5},
    ],
    mark: [
      {"shape":"arc","r":0.86,"a":"1.3+0.4*fire","rot":-1.57,"w":0.08,"stroke":"paper"},
      {"shape":"arc","count":2,"r":"0.8+0.12*i","a":1.2,"rot":3.14,"w":0.05,"stroke":"paper","alpha":"aw","ol":0},
    ],
  },
  // Chrysalis
  Chrysalis: {
    body: [
      {"shape":"line","len":0.34,"w":0.05,"y":-0.5,"stroke":"bone","ol":0},
      {"shape":"group","rot":"0.06*sin(t*1.5)","y":0.05,"layers":[{"shape":"ellipse","rx":0.34,"ry":0.62,"y":0.1,"fill":{"grad":["venom","owner.dark"]}},{"shape":"rect","count":3,"w":0.56,"h":0.06,"y":"-0.15+0.25*i","fill":"ink","ol":0}]},
    ],
    mark: [
      {"shape":"petal","count":2,"len":"0.28+0.35*aw","w":0.22,"x":"pick(i,-0.2,0.2)","y":0,"rot":"pick(i,-1.4,1.4)","fill":"ember","alpha":"0.15+0.3*aw+0.35*beat"},
    ],
  },
  // Trickster
  Trickster: {
    body: [
      {"shape":"petal","len":0.85,"w":0.34,"x":-0.08,"y":0,"rot":-1.15,"fill":"owner"},
      {"shape":"petal","len":0.85,"w":0.34,"x":0.08,"y":0,"rot":1.15,"fill":"blood"},
      {"shape":"circle","count":2,"r":0.11,"x":"pick(i,-0.78,0.78)","y":-0.42,"fill":"gold"},
      {"shape":"circle","r":0.36,"y":0.12,"fill":"paper"},
      {"shape":"circle","count":2,"r":0.05,"x":"pick(i,-0.13,0.13)","y":0.07,"fill":"ink","ol":0},
      {"shape":"arc","r":0.2,"a":1.8,"rot":3.14,"y":0.12,"w":0.05,"stroke":"blood","ol":0},
    ],
    mark: [
      {"shape":"circle","count":2,"r":"0.07+0.04*aw","x":"0.34*cos(t*2+i*3.14)","y":"0.8+0.05*sin(t*2+i*3.14)","fill":"arcane","ol":0},
    ],
  },
  // Gravedigger
  Gravedigger: {
    body: [
      {"shape":"ellipse","rx":0.7,"ry":0.16,"y":0.66,"fill":"owner.dark"},
      {"shape":"group","x":0.3,"y":0.45,"rot":-0.42,"layers":[{"shape":"line","len":1.4,"w":0.12,"stroke":"bone"},{"shape":"circle","r":0.12,"y":-1.4,"fill":"bone"},{"shape":"rect","w":0.46,"h":0.44,"rad":0.15,"y":0.2,"fill":"frost"}]},
    ],
    mark: [
      {"shape":"rect","count":"min(3,1+aw*2)+0","w":0.2,"h":0.28,"rad":0.1,"x":"-0.66+0.2*i","y":0.5,"fill":"bone","alpha":"0.6+0.4*beat"},
    ],
  },
  // Conductor
  Conductor: {
    body: [
      {"shape":"poly","count":2,"n":3,"r":0.5,"x":"pick(i,-0.42,0.42)","y":0.3,"rot":"pick(i,1.571,-1.571)","fill":"night"},
      {"shape":"rect","w":0.2,"h":0.28,"rad":0.04,"y":0.3,"fill":"blood"},
      {"shape":"group","y":0.2,"rot":"0.7*sin(t*3)","layers":[{"shape":"line","len":0.95,"w":0.07,"stroke":"paper"},{"shape":"circle","r":0.08,"y":-0.05,"fill":"bone"}]},
    ],
    mark: [
      {"shape":"circle","count":"3+aw","r":0.08,"x":"-0.55+0.37*i","y":-0.78,"fill":"gold","alpha":"0.4+0.6*beat","ol":0},
    ],
  },
  // Thornback: from the UI duel
  Thornback: {
    body: [
      {"shape":"star","n":13,"r":"0.74+0.16*hit","r2":0.46,"rot":"0.12*sin(t)","fill":{"grad":["bone","owner"]}},
      {"shape":"circle","r":0.4,"fill":"owner.dark"},
      {"shape":"circle","count":2,"r":0.07,"x":"pick(i,-0.13,0.13)","y":-0.06,"fill":"paper","ol":0},
    ],
    mark: [
      {"shape":"group","count":6,"rot":"i*1.047+0.3","layers":[{"shape":"line","len":"0.12+0.2*hit","w":0.05,"y":-0.6,"stroke":"paper","alpha":"0.3+0.7*hit"}]},
      {"shape":"group","count":13,"rot":"i*0.483","alpha":"aw","layers":[{"shape":"line","len":0.16,"w":0.05,"y":-0.78,"stroke":"bone","ol":0}]},
    ],
  },
  // Coin Thief
  "Coin Thief": {
    body: [
      {"shape":"ellipse","rx":0.6,"ry":0.5,"y":0.32,"fill":{"grad":["bone","owner.dark"]}},
      {"shape":"rect","w":0.3,"h":0.34,"rad":0.05,"y":-0.2,"fill":"owner.dark"},
      {"shape":"petal","count":2,"len":0.3,"w":0.13,"y":-0.35,"rot":"pick(i,-0.6,0.6)","fill":"owner.dark"},
      {"shape":"circle","r":0.26,"y":0.3,"fill":"gold"},
      {"shape":"ring","r":0.16,"w":0.04,"y":0.3,"stroke":"ember","ol":0},
    ],
    mark: [
      {"shape":"circle","r":0.1,"x":0.68,"y":"-0.55+0.9*fr(t*0.7)","fill":"gold","alpha":"1-0.7*fr(t*0.7)"},
      {"shape":"circle","r":"0.1*aw","x":-0.68,"y":"-0.55+0.9*fr(t*0.7+0.5)","fill":"gold"},
    ],
  },
  // Lantern: from the UI duel
  Lantern: {
    body: [
      {"shape":"arc","r":0.2,"a":3.4,"y":-0.48,"w":0.08,"stroke":"bone"},
      {"shape":"rect","w":0.64,"h":0.8,"rad":0.14,"y":0.08,"fill":"owner"},
      {"shape":"rect","w":0.4,"h":0.54,"rad":0.08,"y":0.08,"fill":"gold","alpha":"0.8+0.2*sin(t*7)","glow":0.9},
      {"shape":"rect","w":0.5,"h":0.12,"rad":0.03,"y":-0.34,"fill":"ink"},
      {"shape":"rect","w":0.5,"h":0.12,"rad":0.03,"y":0.5,"fill":"ink"},
    ],
    mark: [
      {"shape":"arc","r":0.82,"a":1.2,"rot":-1.571,"w":0.08,"stroke":"frost","alpha":"hp*(0.55+0.45*beat)"},
      {"shape":"arc","r":0.95,"a":1.2,"rot":-1.571,"w":0.06,"stroke":"frost","alpha":"aw*hp*(0.55+0.45*beat)","ol":0},
    ],
  },
  // Hornet: from the UI duel
  Hornet: {
    body: [
      {"shape":"petal","count":2,"len":0.56,"w":0.22,"y":-0.08,"rot":"pick(i,-0.95,0.95)+0.18*sin(t*40+i*3)","fill":"frost","alpha":0.85},
      {"shape":"ellipse","rx":0.27,"ry":0.42,"y":0.15,"fill":"gold"},
      {"shape":"rect","count":2,"w":"pick(i,0.5,0.42)","h":0.09,"y":"0.1+0.2*i","fill":"ink","ol":0},
      {"shape":"poly","n":3,"r":0.12,"y":0.62,"rot":3.14,"fill":"ink"},
      {"shape":"circle","r":0.17,"y":-0.36,"fill":"ink"},
    ],
    mark: [
      {"shape":"circle","count":3,"r":0.07,"x":"0.76*cos(t*1.5+i*2.09)","y":"0.76*sin(t*1.5+i*2.09)","fill":"venom"},
      {"shape":"circle","count":2,"r":0.06,"x":"pick(i,-0.2,0.2)","y":"0.8-0.2*fr(t*0.8+i*0.5)","fill":"venom","alpha":"aw","ol":0},
    ],
  },
  // Prism
  Prism: {
    body: [
      {"shape":"poly","n":4,"r":0.72,"rot":"0.07*sin(t*1.3)","fill":{"grad":["paper","arcane"]}},
      {"shape":"poly","n":4,"r":"0.4+0.05*sin(t*2)","rot":"0.3*sin(t*1.3)","fill":{"radial":["paper","frost"]},"ol":0,"alpha":0.7},
    ],
    mark: [
      {"shape":"line","count":"3+aw","len":0.5,"w":0.06,"x":0.52,"y":0,"rot":"0.9+0.3*i","stroke":"blood","alpha":"0.3+0.7*beat","ol":0},
      {"shape":"line","len":0.5,"w":0.06,"x":-0.62,"y":0.2,"rot":-1.2,"stroke":"paper","ol":0},
    ],
  },
  // War Horn
  "War Horn": {
    body: [
      {"shape":"petal","len":1,"w":0.4,"x":-0.5,"y":0.55,"rot":"1.05+0.05*sin(t*2)","fill":{"grad":["gold","ember"]}},
      {"shape":"circle","r":0.12,"x":-0.5,"y":0.55,"fill":"bone"},
      {"shape":"arc","r":0.2,"a":2.4,"rot":1.05,"x":0.2,"y":-0.15,"w":0.1,"stroke":"gold"},
    ],
    mark: [
      {"shape":"arc","count":"2+aw","r":"0.3+0.17*i","a":0.9,"rot":1.45,"x":0.28,"y":-0.18,"w":0.05,"stroke":"paper","alpha":"0.9-0.25*i","ol":0},
    ],
  },
  // Vulture
  Vulture: {
    body: [
      {"shape":"petal","count":2,"len":0.82,"w":0.36,"x":"pick(i,-0.15,0.15)","y":0.25,"rot":"pick(i,-1.4,1.4)+0.07*sin(t*2)*pick(i,-1,1)","fill":"owner.dark"},
      {"shape":"ellipse","rx":0.3,"ry":0.4,"y":0.28,"fill":"owner"},
      {"shape":"circle","r":0.2,"y":-0.25,"fill":"bone"},
      {"shape":"poly","n":3,"r":0.13,"y":-0.06,"rot":3.14,"fill":"gold"},
      {"shape":"circle","count":2,"r":0.04,"x":"pick(i,-0.08,0.08)","y":-0.3,"fill":"ink","ol":0},
    ],
    mark: [
      {"shape":"line","count":"2+aw","len":0.4,"w":0.06,"x":"-0.2+0.2*i","y":0.8,"rot":"pick(i,0.8,-0.8,0.2)","stroke":"bone","ol":0},
    ],
  },
  // Hexer
  Hexer: {
    body: [
      {"shape":"ellipse","rx":0.88,"ry":0.15,"y":0.4,"fill":"night"},
      {"shape":"poly","n":3,"r":0.62,"y":-0.15,"fill":"night"},
      {"shape":"rect","w":0.52,"h":0.1,"y":0.27,"fill":"gold","ol":0},
      {"shape":"ellipse","rx":0.17,"ry":0.1,"y":-0.08,"fill":"paper","ol":0},
      {"shape":"circle","r":0.06,"x":"0.07*sin(t*1.5)","y":-0.08,"fill":"blood","ol":0},
    ],
    mark: [
      {"shape":"star","n":4,"count":"1+aw","r":0.14,"r2":0.04,"x":"-0.6+1.2*i","y":-0.62,"fill":"arcane","alpha":"0.5+0.5*beat","ol":0},
    ],
  },
  // Fortune Teller
  "Fortune Teller": {
    body: [
      {"shape":"ellipse","count":3,"rx":0.11,"ry":0.32,"x":"pick(i,-0.28,0,0.28)","y":"pick(i,-0.45,-0.58,-0.45)","fill":"bone"},
      {"shape":"petal","count":2,"len":0.42,"w":0.17,"x":"pick(i,-0.35,0.35)","y":0.25,"rot":"pick(i,-1.3,1.3)","fill":"bone"},
      {"shape":"rect","w":0.74,"h":0.78,"rad":0.2,"y":0.18,"fill":{"grad":["bone","owner"]}},
      {"shape":"ellipse","rx":0.25,"ry":0.15,"y":0.18,"fill":"paper"},
      {"shape":"circle","r":0.09,"y":0.18,"fill":"owner.dark","ol":0},
    ],
    mark: [
      {"shape":"circle","r":0.04,"x":0,"y":"0.18","fill":"gold","glow":"beat","ol":0},
      {"shape":"star","n":4,"count":"1+aw","r":0.12,"r2":0.04,"x":"-0.62+1.24*i","y":-0.55,"fill":"gold","ol":0,"alpha":"0.5+0.5*beat"},
    ],
  },
  // Aegis
  Aegis: {
    body: [
      {"shape":"rect","w":0.92,"h":0.84,"rad":0.14,"y":-0.2,"fill":{"grad":["gold","ember"]}},
      {"shape":"poly","n":3,"r":0.56,"y":0.35,"rot":3.14,"fill":{"grad":["ember","gold"]}},
      {"shape":"circle","r":0.3,"y":-0.1,"fill":{"grad":["owner.light","owner.dark"]}},
      {"shape":"star","n":4,"r":"0.15+0.04*beat","r2":0.05,"y":-0.1,"rot":"0.4*sin(t*1.5)","fill":"paper","ol":0},
    ],
    mark: [
      {"shape":"arc","count":"1+aw","r":"0.95+0.12*i","a":1.2,"w":0.05,"y":0,"stroke":"paper","alpha":"0.55+0.45*hit","ol":0},
    ],
  },
  // Gravecaller
  Gravecaller: {
    body: [
      {"shape":"ellipse","rx":0.66,"ry":0.13,"y":0.72,"fill":"owner.dark"},
      {"shape":"rect","w":0.74,"h":1.04,"rad":0.37,"y":0.17,"fill":{"grad":["bone","owner.dark"]}},
      {"shape":"rect","w":0.1,"h":0.42,"rad":0.03,"y":0,"fill":"ink","ol":0},
      {"shape":"rect","w":0.3,"h":0.1,"rad":0.03,"y":-0.06,"fill":"ink","ol":0},
    ],
    mark: [
      {"shape":"petal","count":"1+aw","len":"0.4+0.1*sin(t*3)","w":0.14,"x":"-0.62+0.24*i","y":"0.1-0.2*fr(t*0.5+i*0.5)","fill":"frost","alpha":"0.9-0.6*fr(t*0.5+i*0.5)","ol":0},
    ],
  },
  // Hydra: from the UI duel
  Hydra: {
    body: [
      {"shape":"ellipse","rx":0.5,"ry":0.2,"y":0.58,"fill":"owner.dark"},
      {"shape":"group","count":"3+aw","y":0.5,"rot":"(i-(n-1)/2)*0.55+0.06*sin(t*2+i*2)","layers":[{"shape":"line","len":0.56,"w":0.15,"stroke":"owner"},{"shape":"circle","r":0.17,"y":"-0.63","fill":"owner"},{"shape":"circle","r":0.05,"x":0.06,"y":-0.66,"fill":"gold","ol":0,"glow":0.4}]},
    ],
    mark: [
      {"shape":"poly","count":2,"n":3,"r":0.09,"x":"pick(i,-0.7,0.7)","y":0.2,"rot":3.14,"fill":"paper","alpha":"0.4+0.6*fire"},
    ],
  },
  // Archmage: from the UI duel
  Archmage: {
    body: [
      {"shape":"poly","n":3,"r":0.68,"y":0.08,"fill":{"grad":["arcane","owner.dark"]}},
      {"shape":"ellipse","rx":0.2,"ry":0.12,"y":0.16,"fill":"paper"},
      {"shape":"circle","r":0.07,"x":"0.07*sin(t*1.3)","y":0.16,"fill":"ink","ol":0},
    ],
    mark: [
      {"shape":"circle","count":3,"r":0.1,"x":"0.78*cos(t+i*2.094)","y":"0.78*sin(t+i*2.094)","fill":"arcane","alpha":"0.3+0.7*step(charge,i+1)","glow":"step(charge,i+1)*1.5"},
      {"shape":"star","n":5,"r":0.16,"r2":0.06,"y":-0.64,"fill":"gold","glow":0.6,"alpha":"aw"},
    ],
  },
  // Oracle
  Oracle: {
    body: [
      {"shape":"ellipse","rx":0.88,"ry":0.42,"fill":"paper"},
      {"shape":"circle","r":0.34,"x":"0.1*sin(t*1.5)","fill":{"radial":["owner.light","owner.dark"]}},
      {"shape":"circle","r":"0.15+0.04*beat","x":"0.1*sin(t*1.5)","fill":"ink","ol":0},
      {"shape":"line","count":5,"len":0.2,"w":0.06,"x":"(i-2)*0.3","y":-0.38,"rot":"(i-2)*0.4","stroke":"ink","ol":0},
    ],
    mark: [
      {"shape":"circle","r":0.05,"x":0.1,"y":-0.08,"fill":"paper","ol":0},
      {"shape":"ring","count":"1+aw","r":"0.45+0.14*i","w":0.04,"stroke":"arcane","alpha":"0.4+0.4*beat","ol":0},
    ],
  },
  // Berserker
  Berserker: {
    body: [
      {"shape":"line","len":1.45,"w":0.13,"y":0.8,"stroke":"bone"},
      {"shape":"poly","count":2,"n":3,"r":0.42,"x":"pick(i,-0.3,0.3)","y":-0.42,"rot":"pick(i,-1.571,1.571)","fill":{"grad":["frost","paper"]}},
      {"shape":"circle","r":"0.12+0.05*beat","y":-0.45,"fill":"blood"},
    ],
    mark: [
      {"shape":"line","count":"2+aw","len":0.18,"w":0.06,"x":"pick(i,-0.7,0.7,0.0)","y":"pick(i,0.6,0.6,0.0)","rot":"pick(i,-0.3,0.3,0)","stroke":"blood","alpha":"1-hp","ol":0},
    ],
  },
  // Frostling: from the UI duel
  Frostling: {
    body: [
      {"shape":"group","count":6,"rot":"i*1.047+0.25*t","layers":[{"shape":"line","len":0.72,"w":0.09,"stroke":"frost"},{"shape":"line","count":2,"len":0.22,"w":0.07,"y":-0.44,"rot":"pick(i,-0.75,0.75)","stroke":"frost"}]},
      {"shape":"poly","n":6,"r":0.22,"rot":"0.25*t","fill":"paper"},
    ],
    mark: [
      {"shape":"circle","count":2,"r":0.05,"x":"pick(i,-0.07,0.07)","y":-0.01,"fill":"ink","ol":0},
      {"shape":"circle","count":6,"r":0.05,"x":"0.8*cos(i*1.047+t)","y":"0.8*sin(i*1.047+t)","fill":"frost","alpha":"aw","ol":0},
    ],
  },
  // Priest
  Priest: {
    body: [
      {"shape":"ring","r":0.3,"y":-0.5,"w":0.17,"stroke":"gold"},
      {"shape":"line","len":1,"w":0.17,"y":0.65,"stroke":"gold"},
      {"shape":"line","len":0.85,"w":0.17,"x":-0.43,"y":-0.02,"rot":1.571,"stroke":"gold"},
      {"shape":"circle","r":0.15,"y":0,"fill":"paper"},
    ],
    mark: [
      {"shape":"star","n":4,"count":"2+aw","r":0.13,"r2":0.04,"x":"pick(i,-0.62,0.62,0.0)","y":"pick(i,0.45,0.45,-0.85)","fill":"paper","alpha":"0.4+0.6*beat","ol":0},
    ],
  },
  // Battery
  Battery: {
    body: [
      {"shape":"rect","w":1.14,"h":0.66,"rad":0.12,"x":-0.06,"fill":{"grad":["owner.light","owner.dark"]}},
      {"shape":"rect","w":0.14,"h":0.3,"rad":0.04,"x":0.62,"fill":"bone"},
      {"shape":"rect","count":3,"w":0.2,"h":0.4,"rad":0.04,"x":"-0.34+0.28*i","fill":"gold","alpha":"0.2+0.8*step(charge,i+1)","ol":0},
      {"shape":"star","n":4,"r":"0.16+0.1*beat","r2":0.05,"x":-0.34,"fill":"paper","ol":0,"alpha":"0.9-0.8*step(charge,1)"},
    ],
    mark: [
      {"shape":"star","n":4,"count":"1+aw","r":"0.14+0.06*fire","r2":0.04,"x":"-0.55+1.1*i","y":-0.52,"fill":"gold","glow":"fire","ol":0,"alpha":"0.5+0.5*beat"},
    ],
  },
  // Frost Witch
  "Frost Witch": {
    body: [
      {"shape":"rect","w":1,"h":0.24,"rad":0.06,"y":0.5,"fill":"frost"},
      {"shape":"petal","count":3,"len":"pick(i,0.7,0.95,0.7)","w":0.2,"x":"pick(i,-0.35,0,0.35)","y":0.45,"rot":"pick(i,-0.2,0,0.2)","fill":{"grad":["paper","frost"]}},
      {"shape":"circle","count":3,"r":0.07,"x":"pick(i,-0.35,0,0.35)","y":0.5,"fill":"arcane","ol":0},
    ],
    mark: [
      {"shape":"star","n":6,"count":"3+aw","r":0.09,"r2":0.03,"x":"pick(i,-0.6,0.6,0.0,0.3)","y":"-0.7+0.2*sin(t*2+i)","fill":"paper","alpha":"0.5+0.5*beat","ol":0},
    ],
  },
  // Bully
  Bully: {
    body: [
      {"shape":"rect","w":0.92,"h":0.76,"rad":0.22,"y":0,"fill":{"grad":["owner.light","owner"]}},
      {"shape":"circle","count":4,"r":"0.18+0.02*beat","x":"-0.34+0.227*i","y":"-0.4-0.03*beat","fill":"owner"},
      {"shape":"line","count":3,"len":0.3,"w":0.04,"x":"-0.23+0.227*i","y":-0.14,"rot":3.14,"stroke":"ink","ol":0},
      {"shape":"ellipse","rx":0.34,"ry":0.15,"x":0.02,"y":0.18,"rot":-0.35,"fill":"owner.light"},
      {"shape":"rect","w":0.7,"h":0.26,"rad":0.06,"y":0.68,"fill":"gold"},
    ],
    mark: [
      {"shape":"star","n":5,"r":"0.1+0.2*fire","r2":0.06,"x":0.62,"y":-0.55,"fill":"gold","ol":0,"alpha":"0.4+0.6*fire"},
      {"shape":"line","count":"2+aw","len":0.22,"w":0.06,"x":"-0.7+0.1*i","y":0.7,"rot":1.571,"stroke":"paper","alpha":"0.6","ol":0},
    ],
  },
  // Alchemist
  Alchemist: {
    body: [
      {"shape":"circle","r":0.52,"y":0.3,"fill":{"grad":["venom","owner.dark"]}},
      {"shape":"rect","w":0.3,"h":0.6,"rad":0.05,"y":-0.4,"fill":"frost"},
      {"shape":"rect","w":0.42,"h":0.1,"rad":0.03,"y":-0.74,"fill":"bone"},
      {"shape":"circle","r":0.1,"x":-0.15,"y":0.15,"fill":"paper","alpha":0.6,"ol":0},
    ],
    mark: [
      {"shape":"circle","count":"3+aw","r":0.06,"x":"pick(i,-0.1,0.12,-0.05,0.0)","y":"0.1-0.9*fr(t*0.5+i*0.3)","fill":"venom","alpha":"1-fr(t*0.5+i*0.3)","ol":0},
    ],
  },
  // Executioner
  Executioner: {
    body: [
      {"shape":"rect","count":2,"w":0.14,"h":1.5,"rad":0.04,"x":"pick(i,-0.5,0.5)","y":0.05,"fill":"bone"},
      {"shape":"rect","w":1.14,"h":0.16,"rad":0.04,"y":-0.7,"fill":"bone"},
      {"shape":"rect","w":1.14,"h":0.14,"rad":0.04,"y":0.78,"fill":"owner.dark"},
      {"shape":"poly","n":3,"r":0.4,"y":"-0.38+0.04*sin(t*3)+0.5*fire","rot":3.14,"fill":{"grad":["frost","paper"]}},
    ],
    mark: [
      {"shape":"petal","count":"1+aw","len":0.2,"w":0.1,"x":"-0.14+0.28*i","y":0.55,"rot":3.14,"fill":"blood","alpha":"0.4+0.6*fire","ol":0},
    ],
  },
  // Plague Rat
  "Plague Rat": {
    body: [
      {"shape":"arc","r":0.4,"a":2.8,"rot":1.9,"x":0.5,"y":0.3,"w":0.07,"stroke":"owner.dark"},
      {"shape":"ellipse","rx":0.54,"ry":0.34,"x":0.06,"y":0.22,"fill":{"grad":["bone","owner.dark"]}},
      {"shape":"poly","n":3,"r":0.34,"x":-0.5,"y":0.2,"rot":-1.571,"fill":"bone"},
      {"shape":"circle","count":2,"r":0.13,"x":"pick(i,-0.26,-0.1)","y":-0.1,"fill":"blood"},
      {"shape":"circle","r":0.05,"x":-0.56,"y":0.14,"fill":"ink","ol":0},
    ],
    mark: [
      {"shape":"circle","count":"3+aw","r":0.08,"x":"pick(i,-0.55,-0.1,0.45,0.1)","y":"-0.45-0.2*fr(t*0.6+i*0.25)","fill":"venom","alpha":"0.9-0.5*fr(t*0.6+i*0.25)","ol":0},
    ],
  },
  // Martyr
  Martyr: {
    body: [
      {"shape":"circle","count":2,"r":0.37,"x":"pick(i,-0.28,0.28)","y":-0.2,"fill":{"grad":["blood","owner.dark"]}},
      {"shape":"poly","n":3,"r":0.7,"y":0.02,"rot":3.14,"fill":{"grad":["blood","owner.dark"]}},
      {"shape":"line","len":0.5,"w":0.06,"x":0,"y":0.2,"rot":0.3,"stroke":"paper","ol":0,"alpha":"0.4+0.6*hit"},
    ],
    mark: [
      {"shape":"arc","count":"1+aw","r":"0.8+0.1*i","a":1.4,"rot":3.14,"w":0.06,"y":0,"stroke":"gold","alpha":"0.6+0.4*beat","ol":0},
    ],
  },
  // Ox
  Ox: {
    body: [
      {"shape":"arc","r":0.62,"a":3.4,"rot":"3.14+0.05*sin(t*2)","y":-0.18,"w":0.2,"stroke":"bone"},
      {"shape":"ellipse","rx":0.4,"ry":0.55,"y":0.18,"fill":{"grad":["owner.light","owner.dark"]}},
      {"shape":"ellipse","rx":0.26,"ry":0.18,"y":0.48,"fill":"bone"},
      {"shape":"circle","count":2,"r":0.04,"x":"pick(i,-0.1,0.1)","y":0.48,"fill":"ink","ol":0},
      {"shape":"circle","count":2,"r":0.07,"x":"pick(i,-0.17,0.17)","y":0.05,"fill":"paper"},
    ],
    mark: [
      {"shape":"circle","r":"0.07+0.05*aw+0.02*beat","x":0,"y":0.2,"fill":"blood","ol":0},
    ],
  },
  // Daredevil
  Daredevil: {
    body: [
      {"shape":"circle","count":2,"r":0.28,"x":"pick(i,-0.46,0.46)","y":0.46,"fill":"night"},
      {"shape":"ring","count":2,"r":0.13,"x":"pick(i,-0.46,0.46)","y":0.46,"w":0.05,"stroke":"paper","ol":0},
      {"shape":"line","len":0.95,"w":0.1,"x":-0.46,"y":0.46,"rot":1.31,"stroke":"owner.dark"},
      {"shape":"ellipse","rx":0.34,"ry":0.2,"x":0.05,"y":0.14,"rot":-0.1,"fill":"owner"},
      {"shape":"circle","r":0.2,"x":0.08,"y":-0.16,"fill":"paper"},
      {"shape":"rect","w":0.3,"h":0.1,"rad":0.04,"x":0.12,"y":-0.18,"fill":"ink","ol":0},
      {"shape":"petal","len":"0.3+0.08*sin(t*9)","w":0.13,"x":-0.64,"y":0.48,"rot":-1.571,"fill":{"grad":["ember","gold"]}},
    ],
    mark: [
      {"shape":"star","n":4,"count":"1+aw","r":"0.12+0.1*(1-hp)","r2":0.04,"x":"0.4+0.2*i","y":-0.62,"fill":"gold","ol":0,"alpha":"0.5+0.5*beat"},
    ],
  },
  // Duelist
  Duelist: {
    body: [
      {"shape":"petal","len":1.5,"w":0.2,"x":-0.55,"y":0.55,"rot":"0.785+0.03*sin(t*3)","fill":{"grad":["paper","frost"]}},
      {"shape":"petal","len":1.5,"w":0.2,"x":0.55,"y":0.55,"rot":"-0.785-0.03*sin(t*3)","fill":{"grad":["paper","frost"]}},
      {"shape":"rect","count":2,"w":0.46,"h":0.1,"rad":0.04,"x":"pick(i,-0.38,0.38)","y":0.38,"rot":"pick(i,0.785,-0.785)","fill":"gold"},
      {"shape":"circle","count":2,"r":0.1,"x":"pick(i,-0.62,0.62)","y":0.62,"fill":"bone"},
    ],
    mark: [
      {"shape":"star","n":4,"r":"0.18+0.12*fire+0.1*aw","r2":0.05,"y":-0.05,"fill":"gold","glow":"fire","ol":0,"alpha":"0.6+0.4*beat"},
    ],
  },
  // Paladin
  Paladin: {
    body: [
      {"shape":"petal","count":2,"len":0.8,"w":0.3,"x":"pick(i,-0.1,0.1)","y":0.05,"rot":"pick(i,-1.35,1.35)+0.1*sin(t*3)*pick(i,1,-1)","fill":"paper"},
      {"shape":"petal","count":2,"len":0.55,"w":0.22,"x":"pick(i,-0.1,0.1)","y":0.12,"rot":"pick(i,-1.9,1.9)","fill":"bone"},
      {"shape":"petal","len":1.25,"w":0.2,"y":0.78,"fill":{"grad":["paper","frost"]}},
      {"shape":"rect","w":0.5,"h":0.12,"rad":0.04,"y":0.3,"fill":"gold"},
    ],
    mark: [
      {"shape":"circle","count":"1+aw","r":0.1,"x":"0.0","y":"-0.78+0.26*i","fill":"gold","glow":"0.5+hit","ol":0},
    ],
  },
  // Squire
  Squire: {
    body: [
      {"shape":"petal","len":0.55,"w":0.18,"x":0.2,"y":-0.4,"rot":"0.6+0.15*sin(t*3)","fill":"blood"},
      {"shape":"circle","r":0.5,"y":-0.05,"fill":{"grad":["frost","owner.dark"]}},
      {"shape":"rect","w":0.96,"h":0.4,"rad":0.12,"y":0.5,"fill":"owner.dark"},
      {"shape":"rect","w":0.6,"h":0.09,"rad":0.04,"y":-0.02,"fill":"ink","ol":0},
    ],
    mark: [
      {"shape":"circle","count":"1+aw","r":0.1,"x":"-0.3+0.3*i","y":0.5,"fill":"gold","ol":0,"glow":"shield*0.3"},
    ],
  },
  // Dynamo
  Dynamo: {
    body: [
      {"shape":"group","count":8,"rot":"i*0.785+0.15*t","layers":[{"shape":"rect","w":0.26,"h":0.34,"rad":0.04,"y":-0.64,"fill":"owner.dark"}]},
      {"shape":"circle","r":0.62,"fill":{"grad":["owner.light","owner.dark"]}},
      {"shape":"poly","n":5,"r":0.36,"rot":0.2,"fill":"night"},
    ],
    mark: [
      {"shape":"star","n":3,"count":1,"r":"0.2+0.08*charge","r2":0.06,"rot":0.5,"fill":"gold","glow":"0.4+fire","alpha":"0.6+0.4*beat","ol":0},
      {"shape":"ring","count":"1+aw","r":"0.78+0.1*i","w":0.05,"stroke":"gold","alpha":0.7,"ol":0},
    ],
  },
  // Avenger
  Avenger: {
    body: [
      {"shape":"line","len":1.5,"w":0.12,"x":-0.48,"y":0.78,"stroke":"bone"},
      {"shape":"poly","n":3,"r":0.78,"x":0.04,"y":-0.2,"rot":"1.571+0.07*sin(t*3)","fill":{"grad":["blood","owner.dark"]}},
      {"shape":"star","n":5,"r":0.2,"r2":0.09,"x":0,"y":-0.2,"fill":"paper","ol":0},
    ],
    mark: [
      {"shape":"line","count":"2+aw","len":0.26,"w":0.06,"x":"-0.2+0.2*i","y":0.78,"rot":"pick(i,-0.4,0,0.4,0.2)","stroke":"bone","alpha":"0.5+0.5*fire","ol":0},
    ],
  },
  // Siphon
  Siphon: {
    body: [
      {"shape":"poly","n":3,"r":0.85,"y":-0.15,"rot":3.14,"fill":{"grad":["arcane","owner.dark"]}},
      {"shape":"rect","w":0.18,"h":0.36,"rad":0.04,"y":0.68,"fill":"owner.dark"},
      {"shape":"ellipse","rx":0.5,"ry":0.1,"y":-0.44,"fill":"night","ol":0},
    ],
    mark: [
      {"shape":"arc","count":"2+aw","r":"0.3+0.2*i","a":2.2,"rot":"t*2+i*1.5","y":-0.12,"w":0.05,"stroke":"paper","alpha":"0.9-0.25*i","ol":0},
    ],
  },
  // Comet
  Comet: {
    body: [
      {"shape":"petal","count":4,"len":"0.8+0.1*sin(t*5+i)","w":0.2,"x":0.25,"y":0.25,"rot":"-2.5+(i-1.5)*0.22","fill":{"grad":["frost","owner"]},"alpha":"1-0.12*i"},
      {"shape":"circle","r":0.32,"x":0.25,"y":0.25,"fill":{"radial":["paper","frost"]}},
    ],
    mark: [
      {"shape":"star","n":4,"count":"3+aw","r":0.08,"r2":0.025,"x":"pick(i,-0.6,-0.2,-0.7,-0.45)","y":"pick(i,-0.5,-0.7,0.1,-0.2)","fill":"paper","alpha":"0.4+0.6*beat","ol":0},
    ],
  },
  // Reaper
  Reaper: {
    body: [
      {"shape":"line","len":1.4,"w":0.1,"x":-0.35,"y":0.78,"rot":0.3,"stroke":"bone"},
      {"shape":"arc","r":0.55,"a":1.9,"rot":"1.9+0.04*sin(t*2)","x":0.08,"y":-0.42,"w":0.17,"stroke":"frost"},
      {"shape":"circle","r":0.42,"y":0.15,"fill":{"grad":["owner.dark","ink"]}},
      {"shape":"ellipse","rx":0.27,"ry":0.28,"y":0.17,"fill":"ink","ol":0},
      {"shape":"circle","count":2,"r":0.06,"x":"pick(i,-0.1,0.1)","y":0.15,"fill":"gold","ol":0,"glow":"0.5+beat"},
    ],
    mark: [
      {"shape":"circle","count":"2+aw","r":0.05,"x":"-0.2+0.2*i","y":0.78,"fill":"bone","alpha":"0.4+0.5*beat","ol":0},
    ],
  },
  // Heckler
  Heckler: {
    body: [
      {"shape":"poly","n":3,"r":0.26,"x":-0.38,"y":0.5,"rot":2.6,"fill":"owner"},
      {"shape":"rect","w":1.1,"h":0.8,"rad":0.3,"y":"-0.08-0.03*beat","fill":{"grad":["owner.light","owner"]}},
      {"shape":"rect","w":0.12,"h":0.3,"rad":0.05,"y":"-0.17-0.03*beat","fill":"ink","ol":0},
      {"shape":"circle","r":0.07,"y":"0.12-0.03*beat","fill":"ink","ol":0},
    ],
    mark: [
      {"shape":"arc","count":"2+aw","r":"0.3+0.17*i","a":1,"rot":1.57,"x":0.58,"y":-0.05,"w":0.05,"stroke":"gold","alpha":"0.9-0.25*i","ol":0},
    ],
  },
  // Witch Doctor
  "Witch Doctor": {
    body: [
      {"shape":"petal","count":5,"len":0.7,"w":0.16,"y":-0.2,"rot":"(i-2)*0.45+0.06*sin(t*3+i)","fill":"ember"},
      {"shape":"ellipse","rx":0.42,"ry":0.62,"y":0.1,"fill":{"grad":["owner.light","owner.dark"]}},
      {"shape":"rect","count":2,"w":0.22,"h":0.08,"rad":0.03,"x":"pick(i,-0.17,0.17)","y":-0.05,"rot":"pick(i,0.3,-0.3)","fill":"ink","ol":0},
      {"shape":"rect","w":0.3,"h":0.1,"rad":0.04,"y":0.35,"fill":"bone"},
    ],
    mark: [
      {"shape":"circle","count":3,"r":0.07,"x":"-0.55+0.55*i","y":0.8,"fill":"gold","alpha":"0.25+0.75*step(charge,i+1)","ol":0},
      {"shape":"petal","count":2,"len":0.5,"w":0.14,"x":"pick(i,-0.3,0.3)","y":-0.2,"rot":"pick(i,-1.2,1.2)","fill":"gold","alpha":"aw"},
    ],
  },
  // Broodmother
  Broodmother: {
    body: [
      {"shape":"group","count":8,"rot":"pick(i,-2.4,-1.9,-1.4,-0.9,0.9,1.4,1.9,2.4)+0.08*sin(t*4+i)","y":0.05,"layers":[{"shape":"line","len":0.5,"w":0.07,"stroke":"owner.dark"},{"shape":"line","len":0.4,"w":0.07,"y":-0.46,"rot":0.9,"stroke":"owner.dark"}]},
      {"shape":"ellipse","rx":0.4,"ry":0.5,"y":0.2,"fill":{"grad":["owner.dark","ink"]}},
      {"shape":"circle","r":0.24,"y":-0.28,"fill":"owner"},
      {"shape":"circle","count":2,"r":0.06,"x":"pick(i,-0.09,0.09)","y":-0.32,"fill":"gold","ol":0},
    ],
    mark: [
      {"shape":"circle","count":"1+aw","r":0.1,"x":"-0.14+0.28*i","y":0.3,"fill":"venom","alpha":"0.5+0.5*beat"},
    ],
  },
  // Dragon
  Dragon: {
    body: [
      {"shape":"petal","count":2,"len":0.85,"w":0.4,"x":0.05,"y":0,"rot":"pick(i,-0.5,0.9)+0.08*sin(t*3)*pick(i,1,-1)","fill":"owner.dark"},
      {"shape":"petal","count":3,"len":"0.7+0.1*sin(t*6+i)","w":0.22,"x":0.3,"y":0.1,"rot":"1.2+(i-1)*0.4","fill":{"grad":["gold","ember"]}},
      {"shape":"poly","count":2,"n":3,"r":0.17,"x":"pick(i,-0.2,0.0)","y":-0.34,"rot":"pick(i,-0.5,0.2)","fill":"bone"},
      {"shape":"ellipse","rx":0.46,"ry":0.34,"x":-0.1,"y":0.05,"fill":{"grad":["owner.light","owner.dark"]}},
      {"shape":"poly","n":3,"r":0.32,"x":-0.55,"y":0.12,"rot":-1.571,"fill":"owner"},
      {"shape":"circle","r":0.07,"x":-0.18,"y":-0.05,"fill":"gold","ol":0},
    ],
    mark: [
      {"shape":"circle","count":"2+aw","r":0.07,"x":"0.7+0.1*i","y":"0.1-0.3*fire+0.2*i-0.2","fill":"ember","alpha":"0.4+0.6*fire","glow":"fire","ol":0},
    ],
  },
  // Kraken: from the UI duel
  Kraken: {
    body: [
      {"shape":"line","count":5,"len":"0.52+0.06*sin(t*3+i)","w":0.11,"y":0.12,"rot":"3.14+(i-2)*0.42+0.12*sin(t*2+i)","stroke":"owner.dark"},
      {"shape":"ellipse","rx":0.4,"ry":0.44,"y":-0.2,"fill":{"grad":["owner.light","owner"]}},
      {"shape":"circle","count":2,"r":0.08,"x":"pick(i,-0.15,0.15)","y":-0.1,"fill":"paper"},
      {"shape":"circle","count":2,"r":0.04,"x":"pick(i,-0.15,0.15)","y":-0.08,"fill":"ink","ol":0},
    ],
    mark: [
      {"shape":"arc","count":"min(charge,3)","r":"0.62+0.1*i","a":1,"rot":3.14,"w":0.05,"stroke":"blood","ol":0},
      {"shape":"line","count":2,"len":0.5,"w":0.1,"x":"pick(i,-0.45,0.45)","y":0.2,"rot":"3.14+pick(i,-0.9,0.9)+0.1*sin(t*2+i)","stroke":"owner.dark","alpha":"aw"},
    ],
  },
  // Puppeteer
  Puppeteer: {
    body: [
      {"shape":"rect","w":0.9,"h":0.14,"rad":0.05,"y":-0.7,"fill":"bone"},
      {"shape":"line","count":4,"len":0.62,"w":0.04,"x":"pick(i,-0.45,-0.2,0.2,0.45)","y":-0.65,"rot":"3.14+pick(i,-0.25,-0.08,0.08,0.25)+0.05*sin(t*3+i)","stroke":"paper","ol":0},
      {"shape":"circle","r":0.2,"y":-0.02,"fill":"bone"},
      {"shape":"rect","w":0.46,"h":0.42,"rad":0.1,"y":0.4,"fill":{"grad":["owner","owner.dark"]}},
      {"shape":"line","count":2,"len":0.34,"w":0.12,"x":"pick(i,-0.14,0.14)","y":0.56,"rot":"3.14+pick(i,0.25,-0.25)+0.15*sin(t*3+i*3)","stroke":"owner.dark"},
      {"shape":"line","count":2,"len":0.36,"w":0.1,"x":"pick(i,-0.23,0.23)","y":0.26,"rot":"pick(i,-1.0,1.0)+0.15*sin(t*3+i*2)","stroke":"bone"},
    ],
    mark: [
      {"shape":"circle","count":"1+aw","r":0.06,"x":"-0.3+0.6*i","y":-0.7,"fill":"gold","ol":0},
    ],
  },
  // Phoenix: from the UI duel
  Phoenix: {
    body: [
      {"shape":"petal","count":"5+2*aw","len":"0.72+0.07*sin(t*4+i)","w":0.2,"y":0.4,"rot":"(i-(n-1)/2)*(0.5-0.1*aw)","fill":{"grad":["gold","ember"]}},
      {"shape":"circle","r":0.2,"y":0.36,"fill":"ember"},
      {"shape":"circle","r":0.05,"x":0.06,"y":0.32,"fill":"ink","ol":0},
    ],
    mark: [
      {"shape":"circle","count":4,"r":0.045,"x":"pick(i,-0.42,-0.15,0.18,0.44)","y":"0.62-fr(t*0.6+i*0.27)*1.3","fill":"gold","alpha":"1-fr(t*0.6+i*0.27)","ol":0,"glow":0.6},
    ],
  },
  // Volcano
  Volcano: {
    body: [
      {"shape":"petal","count":3,"len":"0.5+0.1*sin(t*5+i)","w":0.22,"y":-0.3,"rot":"(i-1)*0.5","fill":{"grad":["gold","ember"]}},
      {"shape":"poly","n":3,"r":0.66,"y":0.3,"fill":{"grad":["owner.dark","ink"]}},
      {"shape":"ellipse","rx":0.7,"ry":0.16,"y":0.66,"fill":"owner.dark"},
      {"shape":"ellipse","rx":0.14,"ry":0.06,"y":-0.3,"fill":"ember","ol":0},
    ],
    mark: [
      {"shape":"circle","count":"3+aw","r":0.06,"x":"pick(i,-0.35,0.35,0.0,0.5)*(0.6+fr(t*0.5+i*0.3))","y":"-0.3-0.55*fr(t*0.5+i*0.3)","fill":"ember","alpha":"1-fr(t*0.5+i*0.3)","ol":0},
    ],
  },
  // ---- the summoned bodies (units.ts SUMMONS, units-v2.ts transforms)
  // Imp
  Imp: {
    body: [
      {"shape":"poly","n":3,"r":0.2,"x":0.45,"y":0.62,"rot":1,"fill":"blood"},
      {"shape":"petal","count":2,"len":0.45,"w":0.14,"x":"pick(i,-0.28,0.28)","y":-0.28,"rot":"pick(i,-0.45,0.45)+0.12*beat*pick(i,1,-1)","fill":"bone"},
      {"shape":"circle","r":0.46,"y":0.08,"fill":{"grad":["blood","owner.dark"]}},
      {"shape":"circle","count":2,"r":0.08,"x":"pick(i,-0.17,0.17)","y":0,"fill":"gold","ol":0},
      {"shape":"arc","r":0.18,"a":2.2,"rot":3.14,"y":0.28,"w":0.05,"stroke":"paper","ol":0},
    ],
    mark: [
      {"shape":"star","n":4,"r":"0.1+0.1*fire","r2":0.03,"x":0.6,"y":-0.6,"fill":"ember","ol":0,"alpha":"0.5+0.5*beat"},
      {"shape":"petal","count":2,"len":0.32,"w":0.12,"x":"pick(i,-0.45,0.45)","y":0,"rot":"pick(i,-1.2,1.2)","fill":"blood","alpha":"aw"},
    ],
  },
  // Wolf
  Wolf: {
    body: [
      {"shape":"ellipse","rx":0.54,"ry":0.32,"x":0.1,"y":0.2,"fill":{"grad":["bone","owner.dark"]}},
      {"shape":"petal","len":0.55,"w":0.14,"x":0.56,"y":0.2,"rot":"1.2+0.1*sin(t*4)","fill":"bone"},
      {"shape":"line","count":4,"len":0.4,"w":0.1,"x":"pick(i,-0.3,-0.15,0.3,0.45)","y":0.4,"rot":"3.14+0.12*sin(t*5+i*2)","stroke":"bone"},
      {"shape":"circle","r":0.27,"x":-0.5,"y":-0.05,"fill":"bone"},
      {"shape":"poly","n":3,"r":0.2,"x":-0.72,"y":0,"rot":-1.571,"fill":"paper"},
      {"shape":"poly","n":3,"r":0.12,"x":-0.45,"y":-0.3,"rot":0.2,"fill":"bone"},
      {"shape":"circle","r":0.04,"x":-0.56,"y":-0.08,"fill":"ink","ol":0},
    ],
    mark: [
      {"shape":"arc","r":0.2,"a":1.2,"rot":-1,"x":-0.2,"y":-0.55,"w":0.05,"stroke":"gold","alpha":"0.4+0.6*beat","ol":0},
      {"shape":"arc","count":2,"r":"0.25+0.12*i","a":1,"rot":-1,"x":-0.35,"y":-0.5,"w":0.05,"stroke":"gold","alpha":"aw","ol":0},
    ],
  },
  // Golem
  Golem: {
    body: [
      {"shape":"rect","count":2,"w":0.3,"h":0.7,"rad":0.08,"x":"pick(i,-0.62,0.62)","y":"0.15+0.04*sin(t*3+i*3)","fill":"owner.dark"},
      {"shape":"rect","w":0.84,"h":0.8,"rad":0.12,"y":0.22,"fill":{"grad":["bone","owner.dark"]}},
      {"shape":"rect","w":0.5,"h":0.4,"rad":0.08,"y":"-0.5-0.03*beat","fill":"bone"},
      {"shape":"rect","w":0.34,"h":0.08,"rad":0.02,"y":-0.5,"fill":"ink","ol":0},
      {"shape":"circle","count":2,"r":0.05,"x":"pick(i,-0.1,0.1)","y":-0.5,"fill":"gold","ol":0,"glow":0.5},
    ],
    mark: [
      {"shape":"poly","n":4,"count":"1+aw","r":0.12,"x":"-0.18+0.36*i","y":0.3,"rot":0.3,"fill":"gold","alpha":"0.4+0.5*beat","ol":0},
    ],
  },
  // Wraith
  Wraith: {
    body: [
      {"shape":"petal","len":1,"w":0.3,"x":0.2,"y":0.85,"rot":0.6,"fill":"frost","alpha":0.5},
      {"shape":"circle","r":0.42,"y":-0.2,"fill":{"grad":["paper","arcane"]}},
      {"shape":"rect","w":0.84,"h":0.8,"rad":0.2,"y":0.28,"fill":{"grad":["arcane","night"]}},
      {"shape":"circle","count":3,"r":0.15,"x":"-0.28+0.28*i","y":"0.68+0.04*sin(t*3+i*2)","fill":"night"},
      {"shape":"ellipse","count":2,"rx":0.07,"ry":0.1,"x":"pick(i,-0.15,0.15)","y":-0.2,"fill":"ink","ol":0},
    ],
    mark: [
      {"shape":"circle","count":"2+aw","r":0.05,"x":"-0.6+0.3*i","y":"-0.7+0.1*sin(t*2+i)","fill":"frost","alpha":"0.4+0.5*beat","ol":0},
    ],
  },
  // Treant
  Treant: {
    body: [
      {"shape":"rect","w":0.34,"h":0.9,"rad":0.08,"y":0.4,"fill":{"grad":["owner.dark","ink"]}},
      {"shape":"petal","count":2,"len":0.4,"w":0.12,"x":"pick(i,-0.1,0.1)","y":0.6,"rot":"pick(i,-1.1,1.1)","fill":"owner.dark"},
      {"shape":"circle","count":5,"r":0.34,"x":"pick(i,-0.35,0.0,0.35,-0.18,0.18)","y":"pick(i,-0.22,-0.5,-0.22,-0.1,-0.1)+0.02*sin(t*2+i)","fill":{"grad":["venom","owner"]}},
    ],
    mark: [
      {"shape":"circle","count":"2+aw","r":0.05,"x":"-0.2+0.2*i","y":-0.5,"fill":"gold","alpha":"0.5+0.5*beat","ol":0},
    ],
  },
  // Warg
  Warg: {
    body: [
      {"shape":"star","n":7,"r":0.66,"r2":0.4,"x":0.28,"y":0.02,"rot":"0.1*sin(t*2)","fill":"owner.dark"},
      {"shape":"poly","count":2,"n":3,"r":0.26,"x":"pick(i,-0.1,0.25)","y":-0.5,"rot":"pick(i,-0.2,0.2)","fill":"owner"},
      {"shape":"ellipse","rx":0.5,"ry":0.36,"x":0.05,"y":-0.02,"fill":{"grad":["owner.light","owner"]}},
      {"shape":"poly","n":3,"r":0.46,"x":-0.5,"y":-0.08,"rot":-1.571,"fill":"owner"},
      {"shape":"poly","n":3,"r":0.34,"x":"-0.46","y":"0.26+0.04*beat","rot":"-1.45-0.15*beat","fill":"ink"},
      {"shape":"poly","count":3,"n":3,"r":0.07,"x":"-0.7+0.12*i","y":"0.14","rot":3.14,"fill":"paper","ol":0},
      {"shape":"circle","r":0.07,"x":-0.12,"y":-0.14,"fill":"venom","ol":0,"glow":0.5},
    ],
    mark: [
      {"shape":"petal","count":"1+aw","len":0.22,"w":0.1,"x":"-0.62+0.14*i","y":0.62,"rot":3.14,"fill":"venom","alpha":"0.4+0.6*beat"},
    ],
  },
  // Ghoul
  Ghoul: {
    body: [
      {"shape":"line","count":2,"len":0.7,"w":0.14,"x":"pick(i,-0.18,0.18)","y":0.72,"rot":"pick(i,0.15,-0.15)","stroke":"owner.dark"},
      {"shape":"ellipse","rx":0.34,"ry":0.5,"x":0.05,"y":0.2,"rot":-0.3,"fill":{"grad":["owner","owner.dark"]}},
      {"shape":"circle","r":0.28,"x":-0.1,"y":-0.34,"fill":"bone"},
      {"shape":"line","count":2,"len":0.7,"w":0.12,"x":"pick(i,0.12,0.22)","y":0,"rot":"pick(i,-1.95,-1.6)+0.1*sin(t*3+i)","stroke":"bone"},
      {"shape":"line","count":3,"len":0.2,"w":0.05,"x":-0.65,"y":"-0.14+0.14*i","rot":-1.571,"stroke":"paper","ol":0},
      {"shape":"circle","count":2,"r":0.05,"x":"pick(i,-0.2,0.0)","y":-0.36,"fill":"venom","ol":0},
    ],
    mark: [
      {"shape":"circle","count":"2+aw","r":0.05,"x":"0.5+0.12*i","y":"-0.4+0.1*sin(t*2+i)","fill":"venom","alpha":"0.4+0.5*beat","ol":0},
    ],
  },
  // Puffball
  Puffball: {
    body: [
      {"shape":"circle","count":7,"r":"pick(i,0.38,0.28,0.28,0.26,0.26,0.24,0.24)*(1+0.04*sin(t*2+i))","x":"pick(i,0,-0.42,0.42,-0.3,0.3,-0.55,0.55)","y":"pick(i,0.0,-0.1,-0.1,0.3,0.3,0.25,0.25)","fill":{"grad":["venom","owner.dark"]}},
    ],
    mark: [
      {"shape":"circle","count":"3+aw","r":0.04,"x":"pick(i,-0.5,0.0,0.5,0.25)","y":"-0.5-0.35*fr(t*0.5+i*0.3)","fill":"venom","alpha":"1-fr(t*0.5+i*0.3)","ol":0},
    ],
  },
  // Chick
  Chick: {
    body: [
      {"shape":"line","count":2,"len":0.22,"w":0.07,"x":"pick(i,-0.15,0.15)","y":0.62,"rot":3.14,"stroke":"ember"},
      {"shape":"ellipse","rx":0.5,"ry":0.45,"y":0.3,"fill":{"grad":["gold","ember"]}},
      {"shape":"circle","r":0.3,"x":0.1,"y":-0.22,"fill":"gold"},
      {"shape":"poly","n":3,"r":0.12,"x":0.42,"y":-0.18,"rot":1.571,"fill":"ember"},
      {"shape":"circle","r":0.05,"x":0.14,"y":-0.28,"fill":"ink","ol":0},
      {"shape":"petal","len":0.2,"w":0.08,"x":0.08,"y":-0.5,"rot":"0.1*sin(t*3)","fill":"gold"},
    ],
    mark: [
      {"shape":"circle","count":"1+aw","r":0.06,"x":"-0.6+0.12*i","y":"0.3-0.1*beat","fill":"paper","ol":0},
    ],
  },
  // Head
  Head: {
    body: [
      {"shape":"line","len":0.8,"w":0.3,"x":-0.32,"y":0.78,"rot":"0.4+0.05*sin(t*2)","stroke":"owner.dark"},
      {"shape":"circle","r":0.38,"x":"0.0+0.04*sin(t*2)","y":-0.15,"fill":{"grad":["owner.light","owner"]}},
      {"shape":"poly","n":3,"r":0.24,"x":"0.34+0.04*sin(t*2)","y":-0.1,"rot":1.571,"fill":"owner"},
      {"shape":"poly","count":2,"n":3,"r":0.08,"x":"0.3+0.12*i+0.04*sin(t*2)","y":0.06,"rot":3.14,"fill":"paper","ol":0},
      {"shape":"circle","r":0.07,"x":"0.06+0.04*sin(t*2)","y":-0.24,"fill":"gold","ol":0},
    ],
    mark: [
      {"shape":"poly","n":3,"count":"1+aw","r":0.12,"x":"-0.1+0.2*i","y":-0.72,"rot":"pick(i,-0.4,0.4)","fill":"bone"},
    ],
  },
  // Spiderling
  Spiderling: {
    body: [
      {"shape":"group","count":6,"rot":"pick(i,-2.2,-1.6,-1.0,1.0,1.6,2.2)+0.12*sin(t*5+i)","y":0.05,"layers":[{"shape":"line","len":0.55,"w":0.09,"stroke":"owner.dark"}]},
      {"shape":"circle","r":0.36,"y":0.08,"fill":{"grad":["owner","owner.dark"]}},
      {"shape":"circle","count":2,"r":0.07,"x":"pick(i,-0.12,0.12)","y":-0.02,"fill":"venom","ol":0},
    ],
    mark: [
      {"shape":"circle","r":"0.05+0.03*beat","x":0,"y":0.35,"fill":"venom","ol":0},
      {"shape":"circle","count":2,"r":0.05,"x":"pick(i,-0.2,0.2)","y":-0.1,"fill":"venom","alpha":"aw","ol":0},
    ],
  },
  // Butterfly
  Butterfly: {
    body: [
      {"shape":"petal","count":4,"len":"pick(i,0.85,0.85,0.6,0.6)","w":0.45,"y":0.05,"rot":"pick(i,-1.0,1.0,-2.3,2.3)+0.18*sin(t*6)*pick(i,1,-1,1,-1)","fill":{"grad":["ember","gold"]}},
      {"shape":"ellipse","rx":0.08,"ry":0.4,"y":0.05,"fill":"ink"},
    ],
    mark: [
      {"shape":"circle","count":2,"r":0.07,"x":"pick(i,-0.45,0.45)","y":-0.38,"fill":"paper","ol":0},
      {"shape":"circle","count":4,"r":0.07,"x":"pick(i,-0.45,0.45,-0.35,0.35)","y":"pick(i,-0.2,-0.2,0.35,0.35)","fill":"paper","alpha":"aw","ol":0},
    ],
  },
  // Monarch
  Monarch: {
    body: [
      {"shape":"petal","count":4,"len":"pick(i,0.92,0.92,0.7,0.7)","w":0.5,"y":0.1,"rot":"pick(i,-1.0,1.0,-2.3,2.3)+0.14*sin(t*4)*pick(i,1,-1,1,-1)","fill":{"grad":["blood","ember"]}},
      {"shape":"ellipse","rx":0.09,"ry":0.42,"y":0.1,"fill":"ink"},
    ],
    mark: [
      {"shape":"star","n":5,"r":"0.2+0.05*beat","r2":0.09,"y":-0.62,"fill":"gold","glow":0.5},
      {"shape":"circle","count":"3+aw","r":0.05,"x":"-0.5+0.28*i","y":0.7,"fill":"gold","ol":0},
    ],
  },
};

/** A pool unit's or summoned body's spec by name; undefined for a unit this file does not know (the sigil falls back to the generic one). */
export function portraitOf(name: string): Spec | undefined {
  return Object.hasOwn(PORTRAITS, name) ? PORTRAITS[name] : undefined;
}
