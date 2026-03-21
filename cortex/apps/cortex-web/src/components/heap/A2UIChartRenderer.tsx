import React, { useEffect, useRef } from "react";
import * as d3 from "d3";

export interface A2UIChartData {
    labels: string[];
    datasets: Array<{
        label: string;
        data: number[];
        color?: string;
    }>;
    chart_type?: "bar" | "line" | "area" | "pie";
    title?: string;
}

interface A2UIChartRendererProps {
    data: A2UIChartData;
}

export const A2UIChartRenderer: React.FC<A2UIChartRendererProps> = ({ data }) => {
    const svgRef = useRef<SVGSVGElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const tooltipRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (!svgRef.current || !containerRef.current || !data || !data.datasets || data.datasets.length === 0) return;

        const container = containerRef.current;
        const svg = d3.select(svgRef.current);
        const tooltip = d3.select(tooltipRef.current);

        const render = () => {
            const width = container.clientWidth;
            if (width === 0) return;
            const height = 250;
            const margin = { top: 20, right: 20, bottom: 30, left: 40 };
            const innerWidth = width - margin.left - margin.right;
            const innerHeight = height - margin.top - margin.bottom;

            svg.selectAll("*").remove();
            svg.attr("width", width).attr("height", height);

            const g = svg.append("g").attr("transform", `translate(${margin.left},${margin.top})`);
            const colors = ["#3b82f6", "#8b5cf6", "#10b981", "#ef4444", "#f59e0b"];

            const showTooltip = (event: any, d: number, label: string, datasetLabel: string, color: string) => {
                tooltip
                    .style("opacity", 1)
                    .html(`
                        <div class="flex flex-col gap-1">
                            <div class="text-[10px] font-bold text-slate-400 uppercase tracking-tight">${label}</div>
                            <div class="flex items-center gap-2">
                                <div class="w-2 h-2 rounded-full" style="background-color: ${color}"></div>
                                <div class="text-xs font-semibold text-slate-100">${datasetLabel}: ${d}</div>
                            </div>
                        </div>
                    `)
                    .style("left", (event.pageX - container.getBoundingClientRect().left + 15) + "px")
                    .style("top", (event.pageY - container.getBoundingClientRect().top - 20) + "px");
            };

            const hideTooltip = () => tooltip.style("opacity", 0);

            if (data.chart_type === "pie") {
                const radius = Math.min(innerWidth, innerHeight) / 2;
                const pieG = g.append("g").attr("transform", `translate(${innerWidth / 2},${innerHeight / 2})`);
                const pieData = data.datasets[0].data;

                const pie = d3.pie<number>().value(d => d);
                const arc = d3.arc<d3.PieArcDatum<number>>()
                    .innerRadius(radius * 0.5)
                    .outerRadius(radius * 0.9);

                const arcs = pieG.selectAll(".arc")
                    .data(pie(pieData))
                    .enter()
                    .append("g")
                    .attr("class", "arc");

                arcs.append("path")
                    .attr("d", arc)
                    .attr("fill", (d, i) => data.datasets[0].color || colors[i % colors.length])
                    .attr("stroke", "#0f172a")
                    .attr("stroke-width", 2)
                    .style("opacity", 0.8)
                    .on("mouseover", function (event, d) {
                        d3.select(this).style("opacity", 1).attr("transform", "scale(1.05)");
                        showTooltip(event, d.data, data.labels[d.index], data.datasets[0].label, data.datasets[0].color || colors[d.index % colors.length]);
                    })
                    .on("mousemove", (event) => {
                        tooltip
                            .style("left", (event.pageX - container.getBoundingClientRect().left + 15) + "px")
                            .style("top", (event.pageY - container.getBoundingClientRect().top - 20) + "px");
                    })
                    .on("mouseleave", function () {
                        d3.select(this).style("opacity", 0.8).attr("transform", "scale(1)");
                        hideTooltip();
                    })
                    .transition()
                    .duration(800)
                    .attrTween("d", function (d) {
                        const interpolate = d3.interpolate({ startAngle: 0, endAngle: 0 }, d);
                        return function (t) { return arc(interpolate(t)) as string; };
                    });
                return;
            }

            const allValues = data.datasets.flatMap(d => d.data);
            const yMax = d3.max(allValues) || 100;

            const x = d3.scaleBand().domain(data.labels).range([0, innerWidth]).padding(0.2);
            const y = d3.scaleLinear().domain([0, yMax * 1.1]).range([innerHeight, 0]);

            g.append("g")
                .attr("transform", `translate(0,${innerHeight})`)
                .call(d3.axisBottom(x).tickSizeOuter(0))
                .selectAll("text").style("fill", "#64748b").style("font-family", "mono").style("font-size", "10px");

            g.append("g").call(d3.axisLeft(y).ticks(5))
                .selectAll("text").style("fill", "#64748b").style("font-family", "mono").style("font-size", "10px");

            g.selectAll(".domain, .tick line").style("stroke", "#334155").style("opacity", 0.3);

            const subgroupX = d3.scaleBand()
                .domain(data.datasets.map((_, i) => String(i)))
                .range([0, x.bandwidth()])
                .padding(0.05);

            if (data.chart_type === "bar" || !data.chart_type) {
                data.datasets.forEach((dataset, i) => {
                    g.selectAll(`.bar-${i}`)
                        .data(dataset.data)
                        .enter()
                        .append("rect")
                        .attr("class", `bar-${i}`)
                        .attr("x", (d, j) => (x(data.labels[j]) || 0) + (subgroupX(String(i)) || 0))
                        .attr("y", innerHeight)
                        .attr("width", subgroupX.bandwidth())
                        .attr("height", 0)
                        .attr("fill", dataset.color || colors[i % colors.length])
                        .attr("rx", 2)
                        .on("mouseover", (event, d) => showTooltip(event, d, data.labels[dataset.data.indexOf(d)], dataset.label, dataset.color || colors[i % colors.length]))
                        .on("mouseleave", hideTooltip)
                        .transition().duration(800).attr("y", d => y(d)).attr("height", d => innerHeight - y(d));
                });
            } else if (data.chart_type === "line" || data.chart_type === "area") {
                data.datasets.forEach((dataset, i) => {
                    const color = dataset.color || colors[i % colors.length];

                    if (data.chart_type === "area") {
                        const area = d3.area<number>()
                            .x((d, j) => (x(data.labels[j]) || 0) + x.bandwidth() / 2)
                            .y0(innerHeight)
                            .y1(d => y(d))
                            .curve(d3.curveMonotoneX);

                        g.append("path")
                            .datum(dataset.data)
                            .attr("fill", color)
                            .attr("opacity", 0.2)
                            .attr("d", area);
                    }

                    const line = d3.line<number>()
                        .x((d, j) => (x(data.labels[j]) || 0) + x.bandwidth() / 2)
                        .y(d => y(d))
                        .curve(d3.curveMonotoneX);

                    g.append("path")
                        .datum(dataset.data)
                        .attr("fill", "none")
                        .attr("stroke", color)
                        .attr("stroke-width", 2)
                        .attr("d", line);

                    g.selectAll(`.dot-${i}`)
                        .data(dataset.data)
                        .enter()
                        .append("circle")
                        .attr("cx", (d, j) => (x(data.labels[j]) || 0) + x.bandwidth() / 2)
                        .attr("cy", d => y(d))
                        .attr("r", 4)
                        .attr("fill", color)
                        .attr("stroke", "#0f172a")
                        .attr("stroke-width", 2)
                        .style("opacity", 0)
                        .on("mouseover", (event, d) => {
                            d3.select(event.currentTarget).attr("r", 6).style("opacity", 1);
                            showTooltip(event, d, data.labels[dataset.data.indexOf(d)], dataset.label, color);
                        })
                        .on("mouseleave", (event) => {
                            d3.select(event.currentTarget).attr("r", 4).style("opacity", 0);
                            hideTooltip();
                        });
                });
            }
        };

        render();

        const observer = new ResizeObserver(() => {
            render();
        });
        observer.observe(container);

        return () => observer.disconnect();
    }, [data]);

    return (
        <div className="mt-4 p-4 rounded-xl border bg-slate-900/50 border-slate-700/50 shadow-inner flex flex-col gap-2 group/chart">
            {data.title && <h4 className="text-sm font-semibold text-slate-300">{data.title}</h4>}
            <div ref={containerRef} className="w-full relative overflow-hidden h-[250px]">
                <div
                    ref={tooltipRef}
                    className="absolute z-50 pointer-events-none opacity-0 bg-slate-950/90 backdrop-blur-md border border-slate-700 p-2 rounded-lg shadow-xl transition-opacity duration-200 min-w-[120px]"
                ></div>
                <svg ref={svgRef} className="w-full h-full overflow-visible"></svg>
            </div>
            {data.datasets.length > 0 && (
                <div className="flex flex-wrap gap-3 mt-2 justify-center">
                    {data.datasets.map((ds, i) => (
                        <div key={i} className="flex items-center gap-1.5 text-[10px] font-mono text-slate-400">
                            <span className="w-2.5 h-2.5 rounded-full" style={{ backgroundColor: ds.color || ["#3b82f6", "#8b5cf6", "#10b981", "#ef4444", "#f59e0b"][i % 5] }}></span>
                            {ds.label}
                        </div>
                    ))}
                </div>
            )}
            <div className="text-[10px] text-slate-500 uppercase tracking-widest text-right mt-1 opacity-40 group-hover/chart:opacity-100 transition-opacity">d3 workbench engine</div>
        </div>
    );
};
