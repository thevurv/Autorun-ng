// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

// https://astro.build/config
export default defineConfig({
	site: "https://thevurv.github.io",
	base: "/Autorun-ng/",
	integrations: [
		starlight({
			title: "Autorun",
			social: [
				{
					icon: "github",
					label: "GitHub",
					href: "https://github.com/thevurv/Autorun-ng",
				},
				{
					icon: "discord",
					label: "Discord",
					href: "https://discord.gg/cSC3ebaR3q",
				},
			],
			sidebar: [
				{
					label: "Guides",
					items: [
						{
							label: "Lua Plugins",
							autogenerate: { directory: "guides/lua-plugins" },
						},
						{
							label: "Native Plugins",
							autogenerate: { directory: "guides/native-plugins" },
						},
					],
				},
				{
					label: "Reference",
					items: [
						{
							label: "Developer",
							autogenerate: { directory: "reference/developer" },
						},
						{
							label: "Lua API",
							autogenerate: { directory: "reference/lua-api" },
						},
						{
							label: "Native API",
							autogenerate: { directory: "reference/native-api" },
						},
						{
							label: "Misc",
							autogenerate: { directory: "reference/misc" },
						},
					],
				},
			],
		}),
	],
});
