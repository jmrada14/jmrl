const themeToggle=document.getElementById("theme-toggle"),body=document.body;function initializeTheme(){const e="enabled"===localStorage.getItem("dark-mode");body.classList.toggle("dark-mode",e),themeToggle&&(themeToggle.checked=e)}themeToggle&&themeToggle.addEventListener("change",(()=>{body.classList.toggle("dark-mode"),localStorage.setItem("dark-mode",body.classList.contains("dark-mode")?"enabled":"disabled")})),document.addEventListener("visibilitychange",(()=>{"visible"===document.visibilityState&&initializeTheme()})),window.addEventListener("pageshow",(e=>{e.persisted&&(document.body.style.visibility="visible",initializeTheme())})),document.addEventListener("DOMContentLoaded",(()=>{document.body.style.visibility="visible",initializeTheme()})),window.addEventListener("beforeunload",(()=>{})),document.addEventListener("scroll",(()=>{}),{passive:!0}),initializeTheme();