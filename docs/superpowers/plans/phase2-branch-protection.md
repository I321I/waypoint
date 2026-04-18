# Phase 2：Branch Protection（E2E 必綠）

## 現況

嘗試透過 GitHub API 設 master 的 required status check `e2e-windows`，
但回應 `Upgrade to GitHub Pro or make this repository public to enable this feature.`。

Free tier 的 **private** repo 無法使用 branch protection、也無法建立 rulesets。

## 三個選項

### A. 升級 GitHub Pro（$4/月）
設定後 master 無法直接 push，必須 PR + E2E 綠才能合併。最正規、零維護。

### B. 將 repo 改為 public
免費使用 branch protection / rulesets。需要確認 repo 內沒有敏感資訊
（token、使用者資料）。檢查過 `release.yml` 用 `GITHUB_TOKEN`（runner 自帶），
`CLAUDE.md`、memory 皆無機密。

### C. 不設 protection，改用本地/自律規範
CLAUDE.md 已規定「修正後必須跑 test」，E2E workflow 於 push master 和 PR 都會跑。
流程自律：發現紅燈立刻 revert 或補修。

## 建議

先 **C**（零成本）+ 保留 workflow。若未來誤推壞 code 到 master 造成實際問題，
再考慮 B（最划算）。
