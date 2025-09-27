--[[
	GModPatchTool (formerly GModCEFCodecFix) detection code example

	Copyright 2024-2025, Solstice Game Studios (solsticegamestudios.com)
	LICENSE: GNU General Public License v3.0

	Purpose: Detects if GModPatchTool's CEF patches have been applied successfully on a GMod client.

	Contact:
		Repository: https://github.com/solsticegamestudios/GModPatchTool/
		Discord: https://solsticegamestudios.com/discord/
		Email: contact@solsticegamestudios.com
]]

-- CEF is only on the Client
if not CLIENT then return end

-- Use these global variables for detection elsewhere in your Lua code
CEFAvailable = BRANCH == "x86-64" or system.IsWindows()
CEFCodecFixChecked = false
CEFCodecFixAvailable = false

-- We hook PreRender for reliability
hook.Add("PreRender", "CEFCodecFixCheck", function()
	hook.Remove("PreRender", "CEFCodecFixCheck")

	print("Querying CEF Codec Support...")

	-- If the client isn't on a beta that has CEF, it's impossible for them to have CEFCodecFix
	if not CEFAvailable then
		CEFCodecFixAvailable = false
		CEFCodecFixChecked = true
		print("CEF does not have CEFCodecFix")
		hook.Run("CEFCodecFixStatus", CEFAvailable, CEFCodecFixAvailable)
		return
	end

	local cefTestPanel = vgui.Create("DHTML", nil, "CEFCodecFixCheck")
	cefTestPanel:SetSize(32, 32)
	cefTestPanel:SetKeyboardInputEnabled(false)
	cefTestPanel:SetMouseInputEnabled(false)
	function cefTestPanel:Paint()
		return true -- We don't want this to draw normally
	end
	function cefTestPanel:RemoveWhileHidden()
		-- HACK: The panel apparently draws for a frame once Remove() is called, so we're disabling visibility beforehand
		-- NOTE: Don't use SetVisible(false) to replace the Paint override! Panel Think/Javascript won't run without panel "visibility"
		self:SetVisible(false)
		self:Remove()
	end

	cefTestPanel:SetHTML("")

	function cefTestPanel:OnDocumentReady()
		if not CEFCodecFixChecked then
			self:AddFunction("gmod", "getCodecStatus", function(codecStatus)
				CEFCodecFixAvailable = codecStatus
				CEFCodecFixChecked = true

				if CEFCodecFixAvailable then
					print("CEF has CEFCodecFix")
				else
					print("CEF does not have CEFCodecFix")
				end

				hook.Run("CEFCodecFixStatus", CEFAvailable, CEFCodecFixAvailable)
				self:RemoveWhileHidden()
			end)

			-- This is what actually does the detection, by seeing if the web framework is capable of playing H.264 (a proprietary video codec)
			self:QueueJavascript([[gmod.getCodecStatus(document.createElement("video").canPlayType('video/mp4; codecs="avc1.42E01E, mp4a.40.2"') == "probably")]])
		elseif IsValid(self) then
			self:RemoveWhileHidden()
		end
	end
end)
