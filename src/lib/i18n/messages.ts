import type { LanguageChoice, LimitKind, OAuthStatus } from "../api/usage";
import type { HistoryRange } from "../usage/history";

export type Locale = "en" | "id";

type FallbackStatus = Exclude<OAuthStatus, "disabled" | "active">;

export interface Messages {
  intlLocale: string;
  app: {
    loading: string;
    unavailable: string;
    accurateModeFailed: string;
    dismiss: string;
    settings: string;
    back: string;
  };
  limits: {
    title: Record<LimitKind, string>;
    resetsIn: (countdown: string) => string;
    resettingNow: string;
  };
  countdown: {
    underMinute: string;
    minutes: (minutes: number) => string;
    hoursMinutes: (hours: number, minutes: number) => string;
    daysHours: (days: number, hours: number) => string;
  };
  tokens: (formatted: string, count: number) => string;
  estimate: (percent: string) => string;
  manualEstimate: (percent: string) => string;
  manual: {
    title: string;
    inactive: string;
    save: string;
    clear: string;
    invalid: string;
    failed: string;
    entered: (percent: string, age: string) => string;
  };
  windows: {
    fiveHour: string;
    week: string;
    noUsageFiveHour: string;
    noUsageWeek: string;
  };
  status: {
    estimate: string;
    active: string;
    updatedAgo: (age: string) => string;
    syncing: string;
    reasons: Record<FallbackStatus, string>;
  };
  history: {
    title: string;
    rangeLegend: string;
    choices: Record<HistoryRange, string>;
    rangeNames: Record<HistoryRange, string>;
    noUsage: (range: string) => string;
    summary: (range: string, total: string, peak: string, label: string) => string;
    showData: string;
    hourColumn: string;
    dayColumn: string;
    tokensColumn: string;
  };
  breakdown: {
    title: string;
    groupBy: string;
    model: string;
    project: string;
    other: string;
    noUsage: Record<HistoryRange, string>;
    share: (share: string, tokens: string) => string;
  };
  settings: {
    title: string;
    alertLevels: string;
    warningAt: string;
    highAt: string;
    criticalAt: string;
    refresh: string;
    interval: (minutes: number) => string;
    adaptive: string;
    language: string;
    languageNames: Record<LanguageChoice, string>;
    autostart: string;
    autostartFailed: string;
    save: string;
    saved: string;
    saveFailed: string;
    wholeNumbers: string;
    rising: string;
  };
  accurate: {
    label: string;
    confirmTitle: string;
    risks: {
      token: string;
      storage: string;
      endpoint: string;
      unofficial: string;
      requests: (minutes: number) => string;
    };
    confirm: string;
    cancel: string;
  };
}

export const en: Messages = {
  intlLocale: "en-US",
  app: {
    loading: "Loading usage…",
    unavailable: "Usage is unavailable right now.",
    accurateModeFailed: "Could not change accurate mode. Try again.",
    dismiss: "Dismiss",
    settings: "Settings",
    back: "Back",
  },
  limits: {
    title: { five_hour: "5-hour limit", seven_day: "Weekly limit" },
    resetsIn: (countdown) => `Resets in ${countdown}`,
    resettingNow: "Resetting now",
  },
  countdown: {
    underMinute: "under a minute",
    minutes: (minutes) => `${String(minutes)}m`,
    hoursMinutes: (hours, minutes) => `${String(hours)}h ${String(minutes)}m`,
    daysHours: (days, hours) => `${String(days)}d ${String(hours)}h`,
  },
  tokens: (formatted, count) => `${formatted} ${count === 1 ? "token" : "tokens"}`,
  estimate: (percent) => `≈ ${percent} of limit`,
  manualEstimate: (percent) => `≈ ${percent} of limit · manual`,
  manual: {
    title: "Manual percentage",
    inactive: "Not used while official data is shown",
    save: "Set",
    clear: "Clear",
    invalid: "Enter a number from 0 to 100.",
    failed: "Could not save the manual percentage. Try again.",
    entered: (percent, age) => `${percent} · ${age} ago`,
  },
  windows: {
    fiveHour: "5-hour window",
    week: "This week",
    noUsageFiveHour: "No usage in the last 5 hours.",
    noUsageWeek: "No usage this week yet.",
  },
  status: {
    estimate: "Local estimate",
    active: "Official",
    updatedAgo: (age) => `${age} ago`,
    syncing: "Showing the last official reading while it reconnects",
    reasons: {
      pending: "Connecting…",
      signed_out: "Claude Code signed out",
      token_expired: "Sign-in expired, open Claude Code",
      unauthorized: "Access refused",
      unavailable: "Official data unavailable",
      retrying: "Reconnecting…",
    },
  },
  history: {
    title: "Token history",
    rangeLegend: "Range",
    choices: { day: "24h", week: "7d" },
    rangeNames: { day: "Last 24 hours", week: "Last 7 days" },
    noUsage: (range) => `${range}: no token usage.`,
    summary: (range, total, peak, label) =>
      `${range}: ${total} in total, peak ${peak} at ${label}.`,
    showData: "Show data",
    hourColumn: "Hour",
    dayColumn: "Day",
    tokensColumn: "Tokens",
  },
  breakdown: {
    title: "Breakdown",
    groupBy: "Group by",
    model: "Model",
    project: "Project",
    other: "Other",
    noUsage: { day: "No usage in the last 24 hours.", week: "No usage in the last 7 days." },
    share: (share, tokens) => `${share} · ${tokens} tokens`,
  },
  settings: {
    title: "Settings",
    alertLevels: "Alert levels",
    warningAt: "Warning at",
    highAt: "High at",
    criticalAt: "Critical at",
    refresh: "Refresh official usage",
    interval: (minutes) => (minutes === 1 ? "Every minute" : `Every ${String(minutes)} minutes`),
    adaptive: "Adaptive (recommended)",
    language: "Language",
    languageNames: { system: "Match Windows", en: "English", id: "Bahasa Indonesia" },
    autostart: "Start Plimsoll when Windows starts",
    autostartFailed: "Could not change the startup setting. Try again.",
    save: "Save",
    saved: "Saved.",
    saveFailed: "Could not save settings. Try again.",
    wholeNumbers: "Use whole numbers from 1 to 100.",
    rising: "Each level must be higher than the one before it.",
  },
  accurate: {
    label: "Accurate mode",
    confirmTitle: "Turn on accurate mode?",
    risks: {
      token: "Reads the Claude Code sign-in token on this PC.",
      storage: "The token stays in memory, never saved or sent elsewhere.",
      endpoint: "Uses an undocumented endpoint that may stop working.",
      unofficial: "Not an official Anthropic feature.",
      requests: (minutes) =>
        `Checks ${minutes === 0 ? "every 2 to 30 minutes, faster while you use Claude" : minutes === 1 ? "every minute" : `every ${String(minutes)} minutes`}. Turn off anytime.`,
    },
    confirm: "Turn on",
    cancel: "Cancel",
  },
};

export const id: Messages = {
  intlLocale: "id-ID",
  app: {
    loading: "Memuat pemakaian…",
    unavailable: "Data pemakaian belum tersedia.",
    accurateModeFailed: "Gagal mengubah mode akurat. Coba lagi.",
    dismiss: "Tutup",
    settings: "Pengaturan",
    back: "Kembali",
  },
  limits: {
    title: { five_hour: "Limit 5 jam", seven_day: "Limit mingguan" },
    resetsIn: (countdown) => `Reset dalam ${countdown}`,
    resettingNow: "Sedang reset",
  },
  countdown: {
    underMinute: "kurang dari semenit",
    minutes: (minutes) => `${String(minutes)} menit`,
    hoursMinutes: (hours, minutes) => `${String(hours)} jam ${String(minutes)} menit`,
    daysHours: (days, hours) => `${String(days)} hari ${String(hours)} jam`,
  },
  tokens: (formatted) => `${formatted} token`,
  estimate: (percent) => `≈ ${percent} dari limit`,
  manualEstimate: (percent) => `≈ ${percent} dari limit · manual`,
  manual: {
    title: "Persentase manual",
    inactive: "Tidak dipakai saat data resmi tampil",
    save: "Atur",
    clear: "Hapus",
    invalid: "Isi angka dari 0 sampai 100.",
    failed: "Gagal menyimpan persentase manual. Coba lagi.",
    entered: (percent, age) => `${percent} · ${age} lalu`,
  },
  windows: {
    fiveHour: "Jendela 5 jam",
    week: "Minggu ini",
    noUsageFiveHour: "Belum ada pemakaian dalam 5 jam terakhir.",
    noUsageWeek: "Belum ada pemakaian minggu ini.",
  },
  status: {
    estimate: "Estimasi lokal",
    active: "Resmi",
    updatedAgo: (age) => `${age} lalu`,
    syncing: "Menampilkan data resmi terakhir sambil menghubungkan ulang",
    reasons: {
      pending: "Menghubungkan…",
      signed_out: "Claude Code belum login",
      token_expired: "Login kedaluwarsa, buka Claude Code",
      unauthorized: "Akses ditolak",
      unavailable: "Data resmi tidak tersedia",
      retrying: "Menghubungkan ulang…",
    },
  },
  history: {
    title: "Riwayat token",
    rangeLegend: "Rentang",
    choices: { day: "24 jam", week: "7 hari" },
    rangeNames: { day: "24 jam terakhir", week: "7 hari terakhir" },
    noUsage: (range) => `${range}: tidak ada pemakaian token.`,
    summary: (range, total, peak, label) =>
      `${range}: total ${total}, puncak ${peak} pada ${label}.`,
    showData: "Tampilkan data",
    hourColumn: "Jam",
    dayColumn: "Hari",
    tokensColumn: "Token",
  },
  breakdown: {
    title: "Rincian",
    groupBy: "Kelompokkan menurut",
    model: "Model",
    project: "Proyek",
    other: "Lainnya",
    noUsage: {
      day: "Belum ada pemakaian dalam 24 jam terakhir.",
      week: "Belum ada pemakaian dalam 7 hari terakhir.",
    },
    share: (share, tokens) => `${share} · ${tokens} token`,
  },
  settings: {
    title: "Pengaturan",
    alertLevels: "Level peringatan",
    warningAt: "Peringatan di",
    highAt: "Tinggi di",
    criticalAt: "Kritis di",
    refresh: "Perbarui pemakaian resmi",
    interval: (minutes) => (minutes === 1 ? "Setiap menit" : `Setiap ${String(minutes)} menit`),
    adaptive: "Adaptif (disarankan)",
    language: "Bahasa",
    languageNames: { system: "Ikuti Windows", en: "English", id: "Bahasa Indonesia" },
    autostart: "Jalankan Plimsoll saat Windows dimulai",
    autostartFailed: "Gagal mengubah pengaturan startup. Coba lagi.",
    save: "Simpan",
    saved: "Tersimpan.",
    saveFailed: "Gagal menyimpan pengaturan. Coba lagi.",
    wholeNumbers: "Gunakan bilangan bulat 1 sampai 100.",
    rising: "Setiap level harus lebih tinggi dari level sebelumnya.",
  },
  accurate: {
    label: "Mode akurat",
    confirmTitle: "Aktifkan mode akurat?",
    risks: {
      token: "Membaca token login Claude Code di PC ini.",
      storage: "Token hanya di memori, tidak disimpan atau dikirim ke tempat lain.",
      endpoint: "Memakai endpoint tak terdokumentasi yang bisa berhenti berfungsi.",
      unofficial: "Bukan fitur resmi Anthropic.",
      requests: (minutes) =>
        `Cek ${minutes === 0 ? "tiap 2 sampai 30 menit, lebih sering saat Claude dipakai" : minutes === 1 ? "setiap menit" : `setiap ${String(minutes)} menit`}. Bisa dimatikan kapan saja.`,
    },
    confirm: "Aktifkan",
    cancel: "Batal",
  },
};

const catalogs: Record<Locale, Messages> = { en, id };

export function messagesFor(locale: Locale): Messages {
  return catalogs[locale];
}

export function localeFromTag(tag: string | undefined): Locale {
  return tag?.toLowerCase().split(/[-_]/)[0] === "id" ? "id" : "en";
}
