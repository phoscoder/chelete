import { useEffect, useMemo, useState } from "react";
import { api } from "../../services/api";
import { formatMoney, formatDate } from "../../services/format";
import type { Account, Category, CsvField, CsvMapping, CsvPreview, ImportResult, ImportOptions, ParsedRow } from "../../types";

const FIELD_OPTIONS: { value: CsvField; label: string }[] = [
  { value: "date", label: "Date" },
  { value: "description", label: "Description" },
  { value: "merchant", label: "Merchant" },
  { value: "amount", label: "Amount" },
  { value: "income_amount", label: "Income Amount" },
  { value: "expense_amount", label: "Expense Amount" },
  { value: "type", label: "Type (income/expense)" },
  { value: "category", label: "Category" },
  { value: "account", label: "Account" },
  { value: "currency", label: "Currency" },
  { value: "notes", label: "Notes" },
  { value: "ignore", label: "Ignore" },
];

const DEFAULT_TYPE_ALIASES: Record<string, string> = {
  debit: "expense",
  credit: "income",
  dr: "expense",
  cr: "income",
  in: "income",
  out: "expense",
  expense: "expense",
  income: "income",
};

const MAPPING_STORAGE_KEY = "chelete-csv-mapping";

function buildMappingKey(headers: string[]): string {
  return headers.map((h) => h.trim().toLowerCase().replace(/[^a-z0-9]/g, "")).join(",");
}

function loadSavedMapping(headers: string[]): CsvMapping | null {
  try {
    const raw = localStorage.getItem(MAPPING_STORAGE_KEY);
    if (!raw) return null;
    const data = JSON.parse(raw);
    return data[buildMappingKey(headers)] ?? null;
  } catch {
    return null;
  }
}

function saveMapping(headers: string[], mapping: CsvMapping) {
  try {
    const raw = localStorage.getItem(MAPPING_STORAGE_KEY);
    const data = raw ? JSON.parse(raw) : {};
    data[buildMappingKey(headers)] = mapping;
    localStorage.setItem(MAPPING_STORAGE_KEY, JSON.stringify(data));
  } catch {
    // ignore storage errors
  }
}

function suggestField(header: string): CsvField | undefined {
  const h = header.trim().toLowerCase().replace(/[\s_-]/g, "");
  if (["date", "transactiondate", "transdate", "posted", "datetime"].includes(h)) return "date";
  if (["description", "payee", "memo", "details", "narrative"].includes(h)) return "description";
  if (h === "merchant") return "merchant";
  if (["amount", "value", "sum"].includes(h)) return "amount";
  if (["credit", "deposit", "income", "received", "inflow"].includes(h)) return "income_amount";
  if (["debit", "withdrawal", "payment", "expense", "outflow"].includes(h)) return "expense_amount";
  if (["type", "transactiontype", "kind"].includes(h)) return "type";
  if (["category", "categoryname"].includes(h)) return "category";
  if (["account", "accountname", "source"].includes(h)) return "account";
  if (["currency", "cur"].includes(h)) return "currency";
  if (["notes", "comment", "note"].includes(h)) return "notes";
  return undefined;
}

function buildInitialMapping(headers: string[]): CsvMapping {
  const columns: Record<string, CsvField> = {};
  headers.forEach((h) => {
    const suggested = suggestField(h);
    if (suggested) columns[h] = suggested;
  });
  return {
    columns,
    default_currency: "USD",
    type_aliases: { ...DEFAULT_TYPE_ALIASES },
  };
}

function fieldColumn(mapping: CsvMapping, field: CsvField): string | undefined {
  return Object.entries(mapping.columns).find(([, f]) => f === field)?.[0];
}

function effectiveAccount(row: ParsedRow, mapping: CsvMapping): string | null {
  if (row.account?.trim()) return row.account.trim().toLowerCase();
  if (mapping.default_account_id) return `__default:${mapping.default_account_id}`;
  return null;
}

function isDuplicatePreview(row: ParsedRow, existing: ParsedRow[], mapping: CsvMapping): boolean {
  // Only flag duplicates when there is a meaningful account source. Otherwise rows with
  // blank accounts look identical and all get marked as duplicates.
  if (!row.date || row.amount_cents == null || !row.description) {
    return false;
  }
  const accountA = effectiveAccount(row, mapping);
  if (!accountA) return false;
  return existing.some(
    (r) =>
      r.row_index < row.row_index &&
      r.date === row.date &&
      r.amount_cents === row.amount_cents &&
      r.description?.trim().toLowerCase() === row.description?.trim().toLowerCase() &&
      effectiveAccount(r, mapping) === accountA
  );
}

interface CsvImportDialogProps {
  accounts: Account[];
  categories: Category[];
  onDone: () => void;
  onCancel: () => void;
}

type Stage = "mapping" | "preview" | "result";

export function CsvImportDialog({ accounts, categories, onDone, onCancel }: CsvImportDialogProps) {
  const [stage, setStage] = useState<Stage>("mapping");
  const [path, setPath] = useState<string | null>(null);
  const [mapping, setMapping] = useState<CsvMapping>({
    columns: {},
    default_currency: "USD",
    type_aliases: { ...DEFAULT_TYPE_ALIASES },
  });
  const [preview, setPreview] = useState<CsvPreview | null>(null);
  const [result, setResult] = useState<ImportResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const openFile = async () => {
    setError(null);
    try {
      const selected = await api.openCsvFileDialog();
      if (!selected) return;
      setPath(selected);
      // Load preview with empty mapping to get headers.
      const initialPreview = await api.previewCsvImport(selected, {
        columns: {},
        type_aliases: {},
      });
      const saved = loadSavedMapping(initialPreview.headers);
      const initial = saved ?? buildInitialMapping(initialPreview.headers);
      setMapping(initial);
      const preview = await api.previewCsvImport(selected, initial);
      setPreview(preview);
    } catch (e) {
      setError(String(e));
    }
  };

  useEffect(() => {
    openFile();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const updateColumn = (header: string, field: CsvField | "") => {
    setMapping((prev) => {
      const next: CsvMapping = {
        ...prev,
        columns: { ...prev.columns },
      };
      if (field === "") {
        delete next.columns[header];
      } else {
        // Remove any existing column mapped to the same field so each field is unique.
        Object.keys(next.columns).forEach((h) => {
          if (next.columns[h] === field) {
            delete next.columns[h];
          }
        });
        next.columns[header] = field;
      }
      return next;
    });
  };

  const updateDefaultAccount = (id: string) => {
    setMapping((prev) => ({ ...prev, default_account_id: id || undefined }));
  };

  const updateDefaultCategory = (id: string) => {
    setMapping((prev) => ({ ...prev, default_category_id: id || undefined }));
  };

  const updateCurrency = (currency: string) => {
    setMapping((prev) => ({ ...prev, default_currency: currency.trim().toUpperCase() || "USD" }));
  };

  const updateDefaultType = (type: string) => {
    setMapping((prev) => ({
      ...prev,
      default_transaction_type: type === "" ? undefined : (type as "income" | "expense"),
    }));
  };

  const updateTypeAlias = (from: string, to: string) => {
    setMapping((prev) => ({
      ...prev,
      type_aliases: { ...prev.type_aliases, [from.toLowerCase().trim()]: to },
    }));
  };

  const refreshPreview = async () => {
    if (!path) return;
    setLoading(true);
    setError(null);
    try {
      const preview = await api.previewCsvImport(path, mapping);
      setPreview(preview);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handlePreview = async () => {
    await refreshPreview();
    if (path) {
      const currentPreview = await api.previewCsvImport(path, mapping);
      setPreview(currentPreview);
      saveMapping(currentPreview.headers, mapping);
    }
    setStage("preview");
  };

  const handleImport = async (skipDuplicates: boolean) => {
    if (!path) return;
    setLoading(true);
    setError(null);
    try {
      const options: ImportOptions = {
        default_account_id: mapping.default_account_id,
        default_category_id: mapping.default_category_id,
        default_currency: mapping.default_currency,
        skip_duplicates: skipDuplicates,
      };
      const res = await api.importTransactions(path, mapping, options);
      setResult(res);
      setStage("result");
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const validForPreview = useMemo(() => {
    const hasDate = fieldColumn(mapping, "date");
    const hasDescription = fieldColumn(mapping, "description");
    const hasAmount =
      fieldColumn(mapping, "amount") ||
      fieldColumn(mapping, "income_amount") ||
      fieldColumn(mapping, "expense_amount");
    const hasType =
      fieldColumn(mapping, "type") ||
      fieldColumn(mapping, "income_amount") ||
      fieldColumn(mapping, "expense_amount") ||
      mapping.default_transaction_type;
    return Boolean(hasDate && hasDescription && hasAmount && hasType);
  }, [mapping]);

  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div
        className="modal"
        onClick={(e) => e.stopPropagation()}
        style={{ minWidth: 560, maxWidth: 760, width: "80vw" }}
      >
        <div className="modal-title">Import Transactions from CSV</div>

        {error && (
          <div
            style={{
              background: "var(--chelete-danger)",
              color: "var(--chelete-bg)",
              padding: "10px 12px",
              borderRadius: "var(--chelete-radius)",
              marginBottom: 16,
              fontSize: 13,
            }}
          >
            {error}
          </div>
        )}

        {stage === "mapping" && (
          <MappingStage
            path={path}
            preview={preview}
            mapping={mapping}
            accounts={accounts}
            categories={categories}
            validForPreview={validForPreview}
            onOpenFile={openFile}
            onColumnChange={updateColumn}
            onDefaultAccountChange={updateDefaultAccount}
            onDefaultCategoryChange={updateDefaultCategory}
            onCurrencyChange={updateCurrency}
            onDefaultTypeChange={updateDefaultType}
            onTypeAliasChange={updateTypeAlias}
            onPreview={handlePreview}
            onCancel={onCancel}
            loading={loading}
          />
        )}

        {stage === "preview" && preview && (
          <PreviewStage
            preview={preview}
            mapping={mapping}
            onBack={() => setStage("mapping")}
            onImport={(skip) => handleImport(skip)}
            onCancel={onCancel}
            loading={loading}
          />
        )}

        {stage === "result" && result && (
          <ResultStage result={result} onDone={onDone} onBack={() => setStage("preview")} />
        )}
      </div>
    </div>
  );
}

function MappingStage({
  path,
  preview,
  mapping,
  accounts,
  categories,
  validForPreview,
  onOpenFile,
  onColumnChange,
  onDefaultAccountChange,
  onDefaultCategoryChange,
  onCurrencyChange,
  onDefaultTypeChange,
  onTypeAliasChange,
  onPreview,
  onCancel,
  loading,
}: {
  path: string | null;
  preview: CsvPreview | null;
  mapping: CsvMapping;
  accounts: Account[];
  categories: Category[];
  validForPreview: boolean;
  onOpenFile: () => void;
  onColumnChange: (header: string, field: CsvField | "") => void;
  onDefaultAccountChange: (id: string) => void;
  onDefaultCategoryChange: (id: string) => void;
  onCurrencyChange: (currency: string) => void;
  onDefaultTypeChange: (type: string) => void;
  onTypeAliasChange: (from: string, to: string) => void;
  onPreview: () => void;
  onCancel: () => void;
  loading: boolean;
}) {
  const hasTypeColumn = fieldColumn(mapping, "type") !== undefined;
  const hasSeparateAmounts =
    fieldColumn(mapping, "income_amount") !== undefined ||
    fieldColumn(mapping, "expense_amount") !== undefined;

  return (
    <>
      <div style={{ marginBottom: 16 }}>
        {!path ? (
          <button className="btn btn-primary" onClick={onOpenFile} disabled={loading}>
            Choose CSV File
          </button>
        ) : (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              gap: 12,
              fontSize: 13,
              color: "var(--chelete-fg-muted)",
            }}
          >
            <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
              {path}
            </span>
            <button className="btn" onClick={onOpenFile} disabled={loading}>
              Change
            </button>
          </div>
        )}
      </div>

      {preview && (
        <>
          <div
            style={{
              maxHeight: 320,
              overflow: "auto",
              border: "1px solid var(--chelete-border)",
              borderRadius: "var(--chelete-radius)",
              marginBottom: 16,
            }}
          >
            <table style={{ minWidth: "100%" }}>
              <thead>
                <tr>
                  <th>CSV Column</th>
                  <th>Chelete Field</th>
                </tr>
              </thead>
              <tbody>
                {preview.headers.map((header) => (
                  <tr key={header}>
                    <td>{header}</td>
                    <td>
                      <select
                        className="form-select"
                        value={mapping.columns[header] || ""}
                        onChange={(e) => onColumnChange(header, e.target.value as CsvField | "")}
                        style={{ width: "100%" }}
                      >
                        <option value="">— Ignore —</option>
                        {FIELD_OPTIONS.map((opt) => (
                          <option key={opt.value} value={opt.value}>
                            {opt.label}
                          </option>
                        ))}
                      </select>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12, marginBottom: 16 }}>
            <div className="form-group">
              <label className="form-label">Default Account</label>
              <select
                className="form-select"
                value={mapping.default_account_id || ""}
                onChange={(e) => onDefaultAccountChange(e.target.value)}
              >
                <option value="">Select an account…</option>
                {accounts.map((a) => (
                  <option key={a.id} value={a.id}>
                    {a.name}
                  </option>
                ))}
              </select>
            </div>

            <div className="form-group">
              <label className="form-label">Default Category</label>
              <select
                className="form-select"
                value={mapping.default_category_id || ""}
                onChange={(e) => onDefaultCategoryChange(e.target.value)}
              >
                <option value="">None</option>
                {categories.map((c) => (
                  <option key={c.id} value={c.id}>
                    {c.name} ({c.category_type})
                  </option>
                ))}
              </select>
            </div>

            <div className="form-group">
              <label className="form-label">Default Currency</label>
              <input
                className="form-input"
                type="text"
                value={mapping.default_currency || "USD"}
                onChange={(e) => onCurrencyChange(e.target.value)}
                maxLength={3}
              />
            </div>

            <div className="form-group">
              <label className="form-label">Default Type</label>
              <select
                className="form-select"
                value={mapping.default_transaction_type || ""}
                onChange={(e) => onDefaultTypeChange(e.target.value)}
              >
                <option value="">Infer from columns…</option>
                <option value="expense">Expense</option>
                <option value="income">Income</option>
              </select>
            </div>

            {hasTypeColumn && (
              <div className="form-group">
                <label className="form-label">Type Aliases</label>
                <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                  <AliasRow label="Debit" to="expense" mapping={mapping} onChange={onTypeAliasChange} />
                  <AliasRow label="Credit" to="income" mapping={mapping} onChange={onTypeAliasChange} />
                </div>
              </div>
            )}
          </div>

          {!validForPreview && (
            <div
              style={{
                color: "var(--chelete-warning)",
                fontSize: 13,
                marginBottom: 12,
              }}
            >
              Map at least <strong>Date</strong>, <strong>Description</strong>, and an <strong>Amount</strong> + <strong>Type</strong> field to continue.
              {hasSeparateAmounts ? (
                <>
                  {" "}
                  Separate <strong>Income</strong> and <strong>Expense</strong> amount columns also satisfy the type requirement.
                </>
              ) : null}
            </div>
          )}
        </>
      )}

      <div className="modal-actions">
        <button className="btn" onClick={onCancel} disabled={loading}>
          Cancel
        </button>
        <button
          className="btn btn-primary"
          onClick={onPreview}
          disabled={!path || !validForPreview || loading}
        >
          {loading ? "Loading…" : "Preview"}
        </button>
      </div>
    </>
  );
}

function AliasRow({
  label,
  to,
  mapping,
  onChange,
}: {
  label: string;
  to: string;
  mapping: CsvMapping;
  onChange: (from: string, to: string) => void;
}) {
  const from = label.toLowerCase();
  // `to` is the intended direction shown in the select default; read it as a fallback.
  const value = mapping.type_aliases[from] || to;
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 13 }}>
      <span style={{ color: "var(--chelete-fg-muted)", minWidth: 42 }}>{label} →</span>
      <select
        className="form-select"
        value={value}
        onChange={(e) => onChange(from, e.target.value)}
        style={{ minWidth: 100 }}
      >
        <option value="">—</option>
        <option value="expense">expense</option>
        <option value="income">income</option>
      </select>
    </div>
  );
}

function PreviewStage({
  preview,
  mapping,
  onBack,
  onImport,
  onCancel,
  loading,
}: {
  preview: CsvPreview;
  mapping: CsvMapping;
  onBack: () => void;
  onImport: (skipDuplicates: boolean) => void;
  onCancel: () => void;
  loading: boolean;
}) {
  const [skipDuplicates, setSkipDuplicates] = useState(true);
  const previewWithDupes = useMemo(() => {
    return preview.rows.map((row) => ({
      ...row,
      duplicate: isDuplicatePreview(row, preview.rows, mapping),
    }));
  }, [preview.rows, mapping]);

  const validCount = previewWithDupes.filter((r) => r.errors.length === 0 && !r.duplicate).length;

  return (
    <>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: 12,
          fontSize: 13,
          color: "var(--chelete-fg-muted)",
        }}
      >
        <span>
          {preview.rows.length} preview rows (of {preview.total_rows} total)
        </span>
        <span>
          <strong style={{ color: "var(--chelete-success)" }}>{validCount}</strong> ready to import
        </span>
      </div>

      <div
        style={{
          maxHeight: 360,
          overflow: "auto",
          border: "1px solid var(--chelete-border)",
          borderRadius: "var(--chelete-radius)",
          marginBottom: 16,
        }}
      >
        <table style={{ minWidth: "100%" }}>
          <thead>
            <tr>
              <th>Date</th>
              <th>Description</th>
              <th>Type</th>
              <th style={{ textAlign: "right" }}>Amount</th>
              <th>Account</th>
              <th>Category</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            {previewWithDupes.map((row) => {
              const typeField = fieldColumn(mapping, "type");
              const typeLabel =
                row.transaction_type ||
                (row.amount_source === "income_column" ? "income" : undefined) ||
                (row.amount_source === "expense_column" ? "expense" : undefined) ||
                (typeField ? "—" : "—");
              const accountName = row.account ||
                (mapping.default_account_id ? "(default)" : "—");
              const categoryName = row.category ||
                (mapping.default_category_id ? "(default)" : "—");
              const hasError = row.errors.length > 0;
              const isDup = row.duplicate;

              return (
                <tr key={row.row_index}>
                  <td>{row.date ? formatDate(row.date) : "—"}</td>
                  <td>{row.description || "—"}</td>
                  <td>{typeLabel}</td>
                  <td style={{ textAlign: "right" }}>
                    {row.amount_cents !== undefined ? formatMoney(row.amount_cents) : "—"}
                  </td>
                  <td>{accountName}</td>
                  <td>{categoryName}</td>
                  <td>
                    {hasError ? (
                      <span style={{ color: "var(--chelete-danger)" }}>
                        {row.errors.join("; ")}
                      </span>
                    ) : isDup ? (
                      <span style={{ color: "var(--chelete-warning)" }}>Duplicate</span>
                    ) : (
                      <span style={{ color: "var(--chelete-success)" }}>OK</span>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 16, fontSize: 13 }}>
        <input
          id="skip-duplicates"
          type="checkbox"
          checked={skipDuplicates}
          onChange={(e) => setSkipDuplicates(e.target.checked)}
        />
        <label htmlFor="skip-duplicates" style={{ color: "var(--chelete-fg)" }}>
          Skip exact duplicates (same date, amount, description, account)
        </label>
      </div>

      <div className="modal-actions">
        <button className="btn" onClick={onCancel} disabled={loading}>
          Cancel
        </button>
        <button className="btn" onClick={onBack} disabled={loading}>
          Back
        </button>
        <button
          className="btn btn-primary"
          onClick={() => onImport(skipDuplicates)}
          disabled={validCount === 0 || loading}
        >
          {loading ? "Importing…" : `Import ${validCount} Transactions`}
        </button>
      </div>
    </>
  );
}

function ResultStage({
  result,
  onDone,
  onBack,
}: {
  result: ImportResult;
  onDone: () => void;
  onBack: () => void;
}) {
  return (
    <>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr 1fr",
          gap: 12,
          marginBottom: 16,
        }}
      >
        <div
          style={{
            background: "var(--chelete-surface)",
            borderRadius: "var(--chelete-radius)",
            padding: 16,
            textAlign: "center",
          }}
        >
          <div style={{ fontSize: 24, fontWeight: 600, color: "var(--chelete-success)" }}>
            {result.imported}
          </div>
          <div style={{ fontSize: 12, color: "var(--chelete-fg-muted)", marginTop: 4 }}>
            Imported
          </div>
        </div>
        <div
          style={{
            background: "var(--chelete-surface)",
            borderRadius: "var(--chelete-radius)",
            padding: 16,
            textAlign: "center",
          }}
        >
          <div style={{ fontSize: 24, fontWeight: 600, color: "var(--chelete-warning)" }}>
            {result.skipped}
          </div>
          <div style={{ fontSize: 12, color: "var(--chelete-fg-muted)", marginTop: 4 }}>
            Skipped
          </div>
        </div>
        <div
          style={{
            background: "var(--chelete-surface)",
            borderRadius: "var(--chelete-radius)",
            padding: 16,
            textAlign: "center",
          }}
        >
          <div
            style={{
              fontSize: 24,
              fontWeight: 600,
              color: result.errors.length > 0 ? "var(--chelete-danger)" : "var(--chelete-fg-muted)",
            }}
          >
            {result.errors.length}
          </div>
          <div style={{ fontSize: 12, color: "var(--chelete-fg-muted)", marginTop: 4 }}>
            Errors
          </div>
        </div>
      </div>

      {result.errors.length > 0 && (
        <div
          style={{
            maxHeight: 200,
            overflow: "auto",
            background: "var(--chelete-surface)",
            borderRadius: "var(--chelete-radius)",
            padding: 12,
            marginBottom: 16,
            fontSize: 12,
            color: "var(--chelete-danger)",
          }}
        >
          {result.errors.map((err, i) => (
            <div key={i} style={{ marginBottom: 4 }}>
              {err}
            </div>
          ))}
        </div>
      )}

      <div className="modal-actions">
        <button className="btn" onClick={onBack}>
          Back
        </button>
        <button className="btn btn-primary" onClick={onDone}>
          Done
        </button>
      </div>
    </>
  );
}
