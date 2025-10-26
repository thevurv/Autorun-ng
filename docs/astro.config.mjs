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
							items: [
								{
									label: "Your First Plugin",
									slug: "guides/your-first-plugin",
								},
								{
									label: "Your Second Plugin",
									slug: "guides/your-second-plugin",
								},
							],
						},
						{
							label: "Native Plugins",
							items: [
								{
									label: "Your First Native Plugin",
									slug: "guides/your-first-native-plugin",
								},
							],
						},
					],
				},
				{
					label: "Reference",
					autogenerate: { directory: "reference" },
				},
			],
		}),
	],
});
