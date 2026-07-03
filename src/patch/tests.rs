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

// Regression: a Steam Client Beta dropped MostRecent (and maybe Timestamp) from loginusers.vdf entries (#258)
#[test]
fn loginusers_missing_mostrecent_and_timestamp() {
	let loginusers = "\"users\"\n{\n\t\"76561198059701474\"\n\t{\n\t\t\"AccountName\"\t\"someuser\"\n\t\t\"PersonaName\"\t\"Some User\"\n\t}\n}\n";

	let parsed: HashMap<&str, SteamUser> = vdf::from_str(loginusers).unwrap();
	let user = &parsed["76561198059701474"];
	assert!(!user.most_recent);
	assert_eq!(user.timestamp, 0);
	assert_eq!(user.account_name, "someuser");
}
