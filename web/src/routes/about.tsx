import { createFileRoute } from "@tanstack/react-router";
import { useState, useEffect } from "react";
import { CircleNotch } from "@phosphor-icons/react";
import "github-markdown-css/github-markdown-light.css";

export const Route = createFileRoute("/about")({
  component: AboutPage,
});

const CACHE_KEY = "readme-html";
const CACHE_TTL = 1000 * 60 * 10;
const API_URL =
  "https://api.github.com/repos/nekochanfood/VRCStyledIconMaker/readme";

function getCached(): string | null {
  try {
    const raw = sessionStorage.getItem(CACHE_KEY);
    if (!raw) return null;
    const { html, ts } = JSON.parse(raw);
    if (Date.now() - ts > CACHE_TTL) return null;
    return html;
  } catch {
    return null;
  }
}

function setCache(html: string) {
  try {
    sessionStorage.setItem(CACHE_KEY, JSON.stringify({ html, ts: Date.now() }));
  } catch {}
}

function AboutPage() {
  const [html, setHtml] = useState<string | null>(getCached);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(!html);

  useEffect(() => {
    if (html) return;
    let cancelled = false;

    fetch(API_URL, {
      headers: { Accept: "application/vnd.github.html+json" },
    })
      .then((res) => {
        if (!res.ok) throw new Error(`GitHub API ${res.status}`);
        return res.text();
      })
      .then((text) => {
        if (cancelled) return;
        setCache(text);
        setHtml(text);
      })
      .catch((err) => {
        if (!cancelled) setError(err.message);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [html]);

  if (loading) {
    return (
      <div className="flex items-center justify-center p-12 text-muted-foreground">
        <CircleNotch size={24} className="animate-spin" />
        <span className="ml-2 text-sm">Loading README...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center p-12">
        <p className="text-sm text-destructive">Failed to load README: {error}</p>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-3xl px-4 py-8">
      <div
        className="markdown-body"
        style={{ backgroundColor: "transparent" }}
        dangerouslySetInnerHTML={{ __html: html ?? "" }}
      />
    </div>
  );
}
