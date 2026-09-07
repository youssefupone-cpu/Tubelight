import { describe, expect, it } from "vitest";
import { buildIssueUrl, buildReportText } from "./report";

describe("buildIssueUrl", () => {
	it("points at the Tubelight repo with bug label", () => {
		const url = new URL(
			buildIssueUrl({ title: "[bug] test", error: "boom", route: "#/watch" }),
		);
		expect(url.hostname).toBe("github.com");
		expect(url.pathname).toContain("Tubelight/issues/new");
		expect(url.searchParams.get("labels")).toBe("bug");
		expect(url.searchParams.get("body")).toContain("boom");
	});

	it("never attaches logs automatically", () => {
		const url = new URL(buildIssueUrl({ title: "x" }));
		expect(url.searchParams.get("body")).toContain(
			"never attached automatically",
		);
	});
});

describe("buildReportText", () => {
	it("includes app version and platform", () => {
		const text = buildReportText({ title: "[bug] test", error: "boom" });
		expect(text).toContain("Tubelight v");
		expect(text).toContain("Platform:");
		expect(text).toContain("boom");
	});
});
