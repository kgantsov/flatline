use yew::prelude::*;

#[function_component(LoginPage)]
pub fn login_page() -> Html {
    html! {
        <div style="min-height:100vh;display:flex;flex-direction:column;align-items:center;justify-content:center;padding:24px;background:var(--bg)">
            <div style="background:var(--surface);border:1px solid var(--border);border-radius:var(--radius);padding:40px 36px;width:100%;max-width:380px;display:flex;flex-direction:column;align-items:center;gap:28px">

                // Logo
                <a href="/" style="display:flex;align-items:center;gap:10px;font-weight:700;font-size:18px;letter-spacing:-0.3px;color:var(--text);text-decoration:none">
                    <span class="logo-icon">
                        <svg viewBox="0 0 24 24" fill="none" stroke="#fff" stroke-width="2.5"
                            stroke-linecap="round" stroke-linejoin="round"
                            style="width:16px;height:16px">
                            <polyline points="3 12 6 12 9 4 12 20 15 12 18 12 21 12"/>
                        </svg>
                    </span>
                    { "flatline" }
                </a>

                // Heading
                <div style="text-align:center">
                    <h1 style="font-size:20px;font-weight:600;letter-spacing:-0.3px;margin-bottom:6px">
                        { "Sign in to Flatline" }
                    </h1>
                    <p style="font-size:13.5px;color:var(--text-muted)">
                        { "Use your organization account to continue." }
                    </p>
                </div>

                // OIDC button
                <a href="/auth/login" style="display:flex;align-items:center;justify-content:center;gap:10px;width:100%;padding:11px 20px;background:var(--accent);color:#fff;border-radius:var(--radius);font-size:14px;font-weight:600;cursor:pointer;text-decoration:none;transition:opacity 0.15s;letter-spacing:-0.1px">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
                        stroke-linecap="round" stroke-linejoin="round"
                        style="width:17px;height:17px;flex-shrink:0">
                        <circle cx="12" cy="12" r="10"/>
                        <path d="M12 2a14.5 14.5 0 0 0 0 20M12 2a14.5 14.5 0 0 1 0 20M2 12h20"/>
                    </svg>
                    { "Continue with OIDC" }
                </a>

                <p style="font-size:12px;color:var(--text-muted);text-align:center">
                    { "Access is restricted to authorized users only." }
                </p>
            </div>
        </div>
    }
}
