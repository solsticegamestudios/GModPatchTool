# GModPatchTool <sub>_Formerly GModCEFCodecFix_</sub>

![GModPatchTool](GModPatchToolLogo.png)

***GModPatchTool** does what Facepunch [don't](https://github.com/Facepunch/gmod-html/pull/8)!*

**Created by Solstice Game Studios ([solsticegamestudios.com](https://solsticegamestudios.com))**

# 🛠️ Patches We Apply
### All Platforms
- Fixes various launch/missing main menu issues on macOS and Linux
- Adds `-chromium_fps_max` Launch Option for GMod
  - Allows setting a maximum internal FPS limit for ALL CEF web panels
  - May improve game framerate in exchange for less web content framerate
  - Default is 60
- Adds `-chromium_remote_debugging_port` Launch Option for GMod
  - Opt-in: starts CEF's Remote Debugging server (Chrome DevTools) on the given port so you can inspect in-game HTML panels
  - Off by default; set a port between 1024 and 65535 to enable, e.g. `-chromium_remote_debugging_port 9222`
- Improves the Legacy VGUI Theme with our custom SourceScheme.res
- Replaces Debug/Console fonts with [PT Mono](https://fonts.google.com/specimen/PT+Mono) to improve consistency/readability across platforms
  - This is particularly important for Proton, where text using those fonts is broken/tiny out of the box (no Lucida Console)
  - If you don't like the theme changes or the font replacement, you can disable those patches by using the `--no-sourcescheme` argument when running the tool

### In-Game Web Browser ([Chromium Embedded Framework, aka CEF](https://en.wikipedia.org/wiki/Chromium_Embedded_Framework))
- Updates CEF to 137.0.19 (Chromium 137.0.7151.121)
- Enables [Proprietary Video/Audio codec](https://www.chromium.org/audio-video), like H.264 (MP4) and AAC, support
- Enables [Widevine](https://www.widevine.com) support (but [no VMP](https://github.com/solsticegamestudios/GModPatchTool/issues/100), so Netflix et al. don't work currently...)
- Enables Software WebGL
- Enables partial GPU acceleration
- Improves performance for texture updates
- Disables Hardware Media Keys control of media
- Re-enables Site Isolation (security feature; some sites require it to function)

### Linux
- Can fix Steam Overlay/MangoHud/etc not working
  - Put `GMOD_ENABLE_LD_PRELOAD=1 %command%` in GMod's Launch Options to try it!
  - This is disabled by default because it could just crash GMod instead
- Sets `mesa_glthread=true` for more OpenGL performance with Mesa drivers
- Sets `ulimit -n $(ulimit -Hn)` to fix issues opening/mounting many files (many addons, Lua autorefresh, etc)
- Adds various commented exports to `hl2.sh` to help multi-GPU users quickly point GMod to use the correct GPU (typically Laptops)
  - See [#188](https://github.com/solsticegamestudios/GModPatchTool/issues/188) for why we don't turn these on by default

### macOS
- Pre-warms [Rosetta](https://en.wikipedia.org/wiki/Rosetta_(software)) translations of the patched libraries on Apple Silicon so they don't stall GMod's first launch
  - You can skip this by using the `--skip-rosetta-prewarm` argument when running the tool

# ❓ Players: How to Install / Use
Download the **[Latest Release](https://github.com/solsticegamestudios/GModPatchTool/releases)** and run the application.

Need a more in-depth guide? Take a look at https://solsticegamestudios.com/fixmedia/

# ⚙️ Command Line Options
You don't need any of these to patch normally - just run the tool. But if you want more control:

| Option | What it does |
| --- | --- |
| `-l`, `--launch-gmod` | Launch Garry's Mod after successfully patching |
| `-s`, `--skip-exit-prompt` | Skip "Press Enter to exit..." on tool exit |
| `--steam-path <PATH>` | Force a specific Steam install path (NOT a Steam library path) |
| `--no-sourcescheme` | Don't apply SourceScheme (VGUI Theme) changes |
| `--skip-clear-chromiumcache` | Skip deleting ChromiumCache/ChromiumCacheMultirun/chromium.log from the GarrysMod directory |
| `--skip-rosetta-prewarm` | Skip pre-warming Rosetta translations of patched libraries on Apple Silicon (macOS only) |
| `--disable-cache` | Force redownload all patch files from scratch and clear the GModPatchTool cache directory on exit |
| `--no-system-proxy` | Don't use the OS proxy configuration for HTTP requests - try this if the tool can't download files behind a proxy/VPN |
| `--ignore-gmod-running` | Apply patches even if Garry's Mod is currently running (may cause issues!) |

The tool exits non-zero if patching fails, in case you're scripting around it.

# 👩‍💻 Developers: How to Use / Detect
Direct players to follow the Players' instructions above. This patch is CLIENTSIDE only!

**To Detect Patched CEF:** Check out our [Lua detection example](examples/detection_example.lua).

> [!WARNING]
> Our CEF builds have Site Isolation enabled, which means **you must pay attention to where you're calling JavaScript-related DHTML functions!**
>
> If you call [DHTML.AddFunction](https://wiki.facepunch.com/gmod/DHTML:AddFunction), [DHTML.QueueJavascript](https://wiki.facepunch.com/gmod/DHTML:QueueJavascript), or [DHTML.RunJavascript](https://wiki.facepunch.com/gmod/Panel:RunJavascript) before the page begins loading, it WILL NOT WORK! Make sure you're calling them in [HTML.OnBeginLoadingDocument](https://wiki.facepunch.com/gmod/HTML:OnBeginLoadingDocument) or later.
>
> Site Isolation destroys JavaScript state on navigation like how real web browsers work.
>
> This tool includes a patch for mainmenu.lua that addresses GMod's own issues not using the correct approach, but **this is a breaking change** for any addon that doesn't handle HTML panel states properly for JS.

**If you want to go more in-depth:** Check out [our fork of gmod-html](https://github.com/solsticegamestudios/gmod-html) and [our CEF build scripts](cef_build).

# 📢 Need Help / Contact Us
* Read the FAQ: https://solsticegamestudios.com/fixmedia/faq/
* Discord: https://solsticegamestudios.com/discord/
* Email: contact@solsticegamestudios.com

# 💖 Help Support Us
This project is open source and provided free of charge for the Garry's Mod community.

**If you like what we're doing here, consider [throwing a few dollars our way](https://solsticegamestudios.com/donate/)!** Our work is 100% funded by users of the tool!
