import { describe, expect, it } from "vitest";
import {
	addSubscription,
	cancelJob,
	enqueueJob,
	getSubscriptions,
	likedVideos,
	listJobs,
	pauseJob,
	removeSubscription,
	resumeJob,
	setLiked,
	setWatchLater,
	watchLaterVideos,
} from "./data";

describe("stateful preview store", () => {
	it("toggles subscriptions", () => {
		const before = getSubscriptions().length;
		addSubscription("UCxxx", "Test Channel", null);
		expect(getSubscriptions().length).toBe(before + 1);
		addSubscription("UCxxx", "Test Channel", null);
		expect(getSubscriptions().length).toBe(before + 1);
		removeSubscription("UCxxx");
		expect(getSubscriptions().length).toBe(before);
	});

	it("toggles likes and watch-later", () => {
		const v = { id: "zzz1", title: "t", channel_id: "c", channel_title: "c" };
		setLiked(v, true);
		expect(likedVideos().some((x) => x.id === "zzz1")).toBe(false); // not in catalog
		const real = likedVideos();
		expect(real.length).toBeGreaterThan(0);
		setWatchLater({ ...v, id: "dQw4w9WgXcQ" }, true);
		expect(watchLaterVideos().some((x) => x.id === "dQw4w9WgXcQ")).toBe(true);
	});

	it("enqueues and advances jobs", () => {
		const id = enqueueJob("dQw4w9WgXcQ", "22", "/tmp");
		const jobs = listJobs();
		expect(jobs.some((j) => j.id === id)).toBe(true);
		pauseJob(id);
		expect(listJobs().find((j) => j.id === id)?.state).toBe("Queued"); // pause only affects Running
		resumeJob(id);
		expect(listJobs().find((j) => j.id === id)?.state).toBe("Running");
		pauseJob(id);
		expect(listJobs().find((j) => j.id === id)?.state).toBe("Paused");
		cancelJob(id);
		expect(listJobs().find((j) => j.id === id)?.state).toBe("Cancelled");
	});
});
