use super::*;

// TODO: Flesh out many more tests

// Regression: unbalanced bracket inside a quoted value (e.g. Lua code in an Overlay page title) overran the section end
#[test]
fn strip_localconfig_webstorage_unbalanced_bracket_in_value() {
	let localconfig = r#""UserLocalConfigStore"
{
	"WebStorage"
	{
		"OverlaySavedDataV2_334920_webrequests"		"{\"strTitle\":\"function widget:GetInfo() return { name\"}"
		"SomePath"		"C:\\Users\\"
	}
	"Software"
	{
		"Valve"
		{
			"Steam"
			{
				"apps"
				{
					"4000"
					{
						"LaunchOptions"		"-test"
					}
				}
			}
		}
	}
}
"#;

	let stripped = strip_localconfig_webstorage(localconfig.to_string());
	assert!(stripped.contains("\"WebStorage\"\n\t{}"));

	let parsed: SteamUserLocalConfig = vdf::from_str(&stripped).unwrap();
	assert_eq!(parsed.software.valve.steam.apps.gmod.unwrap().launch_options.as_deref(), Some("-test"));
}

#[test]
fn strip_localconfig_webstorage_no_section() {
	let localconfig = "\"UserLocalConfigStore\"\n{\n}\n";
	assert_eq!(strip_localconfig_webstorage(localconfig.to_string()), localconfig);
}
