# UI Unification Progress

## Steps

- [x] 1. Create Theme file with all colors
- [x] 2. Replace hardcoded colors in components
- [x] 3. Replace hardcoded colors in modals
- [x] 4. Replace hardcoded colors in pages
- [ ] 5. Remaining: widgets.slint, sidebar.slint, charts.slint, settings.slint
- [ ] 6. Final cleanup and verification

---

## Progress Log

### Step 1: Theme File Created
Created `ui/theme.slint` with organized color tokens:
- Core colors (bg, bg-surface, bg-hover, bg-sidebar)
- Text colors (text, text-secondary, text-muted, text-subtle)
- Border colors (border, border-light, border-subtle)
- Accent/Primary colors
- Semantic colors (success, danger, warning)
- Overlays and shadows (overlay, overlay-strong, overlay-soft, shadow)
- Chart colors (chart-bg-start, chart-bg-end, chart-line)
- Notification colors (notify-error-bg/border, notify-success-bg/border)
- Icon backgrounds (icon-error-bg/border, icon-success-bg/border)
- Account type colors
- Transaction colors + toggle colors
- Modal colors
- Heatmap colors
- Habit colors (16 + gradient)
- Size tokens (radius, icons, spacing, fonts)

### Step 2: Components Updated
- account_item.slint: Using Theme for icons, gradients, radius
- transaction_item.slint: Using Theme for category icons, delete button
- notification.slint: Using Theme for backgrounds, borders, icons
- habit_heatmap.slint: Using Theme for tooltip

### Step 3: Modals Updated
- add_transaction.slint: Theme for overlay, shadows, toggles, button text
- add_habit.slint: Theme for overlay, shadows, gradients, button text
- add_account.slint: Theme for overlay, shadows, button text
- transfer_funds.slint: Theme for overlay, shadows, button text
- configure_categories.slint: Theme for overlay, shadows
- configure_ticker.slint: Theme for overlay, checkbox text

### Step 4: Pages Updated
- finances.slint: Theme for shadows, overlay, gradient
- habits.slint: Theme for gradient

### Step 5: Remaining
Still have hardcoded colors in:
- widgets.slint (button text #ffffff)
- sidebar.slint (text color)
- charts.slint (gradients and strokes)
- settings.slint (gradients)
- category_breakdown.slint (gradient)

### Verification
- cargo check: PASSED
