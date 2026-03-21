import { expect, test } from "@playwright/test";

const LAYOUT_SPEC_FIXTURE = {
    layoutId: "shell_layout_v2",
    navigationGraph: {
        entries: [
            { routeId: "/heap", label: "Heap Canvas", icon: "HP", category: "Core", requiredRole: "viewer" }
        ]
    }
};

const HEAP_WORKBENCH_FIXTURE = {
    type: "surface",
    surfaceId: "surface.heap.enrichment",
    title: "Heap Enrichment Test",
    meta: {},
    components: [
        {
            id: "heap_canvas",
            type: "Container",
            props: { widgetType: "HeapCanvas" },
            children: []
        }
    ]
};

const HEAP_BLOCKS_FIXTURE = {
    schemaVersion: "1.0.0",
    generatedAt: "2026-03-10T00:00:00Z",
    count: 7,
    hasMore: false,
    items: [
        {
            projection: {
                artifactId: "mock-chart-1",
                title: "Agent Performance Metrics",
                blockType: "chart",
                updatedAt: "2026-03-10T00:00:00Z"
            },
            surfaceJson: {
                payload_type: "chart",
                tree: {
                    chart_type: "line",
                    title: "Response Time (ms)",
                    labels: ["08:00", "09:00"],
                    datasets: [{ label: "Model A", data: [1200, 1300], color: "#3b82f6" }]
                }
            }
        },
        {
            projection: {
                artifactId: "mock-activity-1",
                title: "Deployment Event Log",
                blockType: "telemetry",
                updatedAt: "2026-03-10T00:00:00Z"
            },
            surfaceJson: {
                payload_type: "telemetry",
                tree: {
                    widget: "ActivityFeed",
                    title: "System Events",
                    items: [{ action: "Started", detail: "Rollout", timestamp: "2026-03-10T07:45:00Z" }]
                }
            }
        },
        {
            projection: {
                artifactId: "mock-scorecard-1",
                title: "Security Compliance Audit",
                blockType: "widget",
                updatedAt: "2026-03-10T00:00:00Z"
            },
            surfaceJson: {
                payload_type: "a2ui",
                tree: {
                    widget: "SiqScorecard",
                    passing: false,
                    score: 72,
                    violations: [{ node: "CortexGateway", error: "Missing token rotation" }]
                }
            }
        },
        {
            projection: {
                artifactId: "mock-gate-1",
                title: "Infrastructure Gate Summary",
                blockType: "structured_data",
                updatedAt: "2026-03-10T00:00:00Z"
            },
            surfaceJson: {
                payload_type: "structured_data",
                structured_data: {
                    schema_id: "nostra.heap.block.gate_summary.v1",
                    overall_verdict: "FAILED",
                    counts: { total_tests: 100, passed: 97, failed: 3, skipped: 0 }
                }
            }
        },
        {
            projection: {
                artifactId: "mock-media-1",
                title: "Architecture Diagram Draft",
                blockType: "media",
                updatedAt: "2026-03-10T00:00:00Z"
            },
            surfaceJson: {
                payload_type: "media",
                media: {
                    url: "https://example.com/image.png",
                    mime_type: "image/png"
                }
            }
        },
        {
            projection: {
                artifactId: "mock-solicitation-1",
                title: "Pending Agent Proposal",
                blockType: "widget",
                updatedAt: "2026-03-10T00:00:00Z"
            },
            surfaceJson: {
                payload_type: "a2ui",
                tree: {
                    widget: "AgentBenchmarkRecord",
                    agent_role: "steward.security",
                    rationale: "Restrict egress traffic"
                }
            }
        },
        {
            projection: {
                artifactId: "mock-task-1",
                title: "Release Preparation Checklist",
                blockType: "task",
                updatedAt: "2026-03-10T00:00:00Z"
            },
            surfaceJson: {
                payload_type: "task",
                text: "### Checklist\n- [x] Done\n- [ ] Pending"
            }
        }
    ]
};

test.beforeEach(async ({ page }) => {
    await page.route("**/api/cortex/layout/spec", async (route) => {
        await route.fulfill({ status: 200, body: JSON.stringify(LAYOUT_SPEC_FIXTURE) });
    });
    await page.route("**/api/system/ux/workbench**", async (route) => {
        await route.fulfill({ status: 200, body: JSON.stringify(HEAP_WORKBENCH_FIXTURE) });
    });
    await page.route("**/api/cortex/studio/heap/blocks**", async (route) => {
        await route.fulfill({ status: 200, body: JSON.stringify(HEAP_BLOCKS_FIXTURE) });
    });
    await page.route("**/api/cortex/studio/heap/changed_blocks**", async (route) => {
        await route.fulfill({ 
            status: 200, 
            body: JSON.stringify({ 
                schemaVersion: "1.0.0",
                generatedAt: new Date().toISOString(),
                count: 0, 
                hasMore: false,
                changed: [], 
                deleted: [] 
            }) 
        });
    });
});

test("verifies all enriched block types render correctly", async ({ page }) => {
    await page.goto("/heap");

    // 1. Chart
    const chartCard = page.locator(".heap-block-card").filter({ hasText: "Agent Performance Metrics" });
    await expect(chartCard).toBeVisible();
    await expect(chartCard.locator("text=Response Time (ms)")).toBeVisible();

    // 2. Activity Feed
    const activityCard = page.locator(".heap-block-card").filter({ hasText: "Deployment Event Log" });
    await expect(activityCard).toBeVisible();
    await expect(activityCard).toContainText("System Events");
    await expect(activityCard).toContainText(/rollout/i);

    // 3. SIQ Scorecard
    const scorecardCard = page.locator(".heap-block-card").filter({ hasText: "Security Compliance Audit" });
    await expect(scorecardCard).toBeVisible();
    await expect(scorecardCard).toContainText("72/100");
    await expect(scorecardCard).toContainText("CortexGateway");

    // 4. Gate Summary
    const gateCard = page.locator(".heap-block-card").filter({ hasText: "Infrastructure Gate Summary" });
    await expect(gateCard).toBeVisible();
    await expect(gateCard).toContainText("FAILED");
    await expect(gateCard).toContainText("passed97");

    // 5. Media
    const mediaCard = page.locator(".heap-block-card").filter({ hasText: "Architecture Diagram Draft" });
    await expect(mediaCard).toBeVisible();
    await expect(mediaCard.locator("img")).toHaveAttribute("src", /example\.com/);

    // 6. Solicitation
    const solicitationCard = page.locator(".heap-block-card").filter({ hasText: "Pending Agent Proposal" });
    await expect(solicitationCard).toBeVisible();
    await expect(solicitationCard).toContainText("steward.security");
    await expect(solicitationCard).toContainText("Restrict egress traffic");

    // 7. Task
    const taskCard = page.locator(".heap-block-card").filter({ hasText: "Release Preparation Checklist" });
    await expect(taskCard).toBeVisible();
    await expect(taskCard.locator('input[type="checkbox"]')).toHaveCount(2);
});
