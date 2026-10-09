# Chelete Roadmap

This is a living list of features and improvements planned for Chelete. Items are grouped by theme and roughly ordered by priority.

## Now

Features that build directly on what exists today.

- [ ] **Budgets / spending limits per category**
  - Set monthly or custom-period budgets for expense categories.
  - Show progress bars and warnings on Overview and Categories pages.

- [x] **Ability to mark subscriptions as active/inactive**
   - Show a green circle to mark if a subscription is active 
   - When I edit a subscription show a dropdown to mark a subscription as active or inactive

- [ ] **Subscription reminders**
  - Surface "due soon" subscriptions on the Overview dashboard.
  - One-click "Pay now" to create a real transaction from a subscription.

- [ ] **Transaction search**
  - Search Recent Transactions by description, merchant, category, or amount.
  - Done: description, merchant, type, category and account. Still to do: amount.

- [x] **Auto calculate charges for Ecocash**
  - For payments and tranfers done using Ecocash specify amount spend and left amount then auto calculate charges

- [x] **Protect account deletion**
  - If a user press delete account prompt them to confirm before deletion

- [x] **Correct account amounts**
  - Add an Edit button besides the delete account which should allow editing account amounts 

## Next

Features that add more insight and automation.

- [ ] **Income vs. Expenses over time**
  - Line or bar chart of monthly income and expenses.

- [ ] **Net worth tracking**
  - Track total balance over time and visualize it as a line chart.

- [ ] **Category trends**
  - View spending per category across the last 6–12 months.

- [x] **Import transactions**
  - Upload CSV or bank statements and map columns to Chelete fields.
  - Detect duplicates during import.

## Later

Bigger features and platform-level improvements.

- [x] **Export data**
  - Export transactions, accounts, categories, and subscriptions to CSV/JSON.

- [ ] **Backup / restore**
  - Download and restore the SQLite database file.

- [ ] **Transfers between accounts**
  - A transaction type that moves money between two accounts without affecting income/expense totals.

- [ ] **Split transactions**
  - Allocate one transaction across multiple categories.

- [ ] **Transaction tags or payees**
  - Flexible grouping beyond categories.

- [ ] **Desktop notifications**
  - Tauri notifications for upcoming bills and subscription due dates.

- [ ] **Keyboard shortcut customization**
  - Remap shortcuts in Settings.

- [ ] **Onboarding / empty states**
  - Guided setup for first-time users.

## Under consideration

Ideas that need more research or may not fit the current scope.

- [ ] Multi-user support
- [ ] macOS / Windows builds
- [ ] Multi-currency support with exchange rates
- [ ] Receipt attachments

---

Want to pick the next item? Open an issue or jump into the code and check the box above.
