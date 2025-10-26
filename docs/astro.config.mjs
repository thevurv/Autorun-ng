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
						{ label: "Your First Plugin", slug: "guides/your-first-plugin" },
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
