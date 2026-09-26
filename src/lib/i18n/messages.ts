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
    used: string;
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
  estimate: (percent: string, samples: number) => string;
  manualEstimate: (percent: string, entered: string, age: string) => string;
  manual: {
    title: string;
    help: string;
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
    showingLast: string;
    showingEstimates: string;
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
    alertHelp: string;
    warningAt: string;
    highAt: string;
    criticalAt: string;
    refresh: string;
    refreshHelp: string;
    interval: (minutes: number) => string;
    language: string;
    languageNames: Record<LanguageChoice, string>;
    save: string;
    saved: string;
    saveFailed: string;
    wholeNumbers: string;
    rising: string;
  };
  accurate: {
    label: string;
    help: string;
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
    used: "used",
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
  estimate: (percent, samples) =>
    `About ${percent} of the limit, estimated from ${String(samples)} past windows`,
  manualEstimate: (percent, entered, age) =>
    `About ${percent} of the limit, based on your entry of ${entered} ${age} ago`,
  manual: {
    title: "Manual percentage",
    help: "When official data is unavailable, enter the percentage shown by /usage in Claude Code or on claude.ai. It is used until that limit could have reset.",
    save: "Set",
    clear: "Clear",
    invalid: "Enter a number from 0 to 100.",
    failed: "Could not save the manual percentage. Try again.",
    entered: (percent, age) => `Entered ${percent}, ${age} ago`,
  },
  windows: {
    fiveHour: "5-hour window",
    week: "This week",
    noUsageFiveHour: "No usage in the last 5 hours.",
    noUsageWeek: "No usage this week yet.",
  },
  status: {
    estimate:
      "Estimated from local Claude Code logs, without chat, desktop or mobile usage. Turn on accurate mode in Settings for official percentages.",
    active: "Official usage from your Claude account.",
    showingLast: "Showing the last official reading.",
    showingEstimates: "Showing local estimates.",
    reasons: {
      pending: "Connecting to your Claude account.",
      signed_out: "Claude Code is not signed in on this PC.",
      token_expired: "The Claude Code sign-in has expired. Open Claude Code to refresh it.",
      unauthorized: "Your Claude account refused the request.",
      unavailable: "Official usage is unavailable right now.",
      retrying: "Could not reach Claude. Retrying soon.",
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
    alertHelp: "The tray color changes at these percentages of a limit.",
    warningAt: "Warning at",
    highAt: "High at",
    criticalAt: "Critical at",
    refresh: "Refresh official usage",
    refreshHelp: "Used while accurate mode is on.",
    interval: (minutes) => (minutes === 1 ? "Every minute" : `Every ${String(minutes)} minutes`),
    language: "Language",
    languageNames: { system: "Match Windows", en: "English", id: "Bahasa Indonesia" },
    save: "Save",
    saved: "Saved.",
    saveFailed: "Could not save settings. Try again.",
    wholeNumbers: "Use whole numbers from 1 to 100.",
    rising: "Each level must be higher than the one before it.",
  },
  accurate: {
    label: "Accurate mode",
    help: "Reads the Claude Code sign-in on this PC to show official percentages from Anthropic. The token stays in memory and is never saved or sent anywhere else.",
    confirmTitle: "Before you turn on accurate mode",
    risks: {
      token:
        "Plimsoll reads the sign-in token that Claude Code saved on this PC and uses it only to ask Anthropic for your usage.",
      storage:
        "The token stays in memory. It is never written to disk, logged or sent anywhere else.",
      endpoint:
        "The usage endpoint is undocumented. Anthropic may change or block it at any time, and Plimsoll then falls back to local estimates.",
      unofficial:
        "This is not an official Anthropic feature. Turn it on only if you are comfortable with that.",
      requests: (minutes) =>
        `While it is on, Plimsoll asks for your usage ${minutes === 1 ? "once a minute" : `every ${String(minutes)} minutes`}. You can turn it off at any time.`,
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
    used: "terpakai",
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
  estimate: (percent, samples) =>
    `Sekitar ${percent} dari limit, estimasi dari ${String(samples)} jendela sebelumnya`,
  manualEstimate: (percent, entered, age) =>
    `Sekitar ${percent} dari limit, dari isian manual ${entered} ${age} lalu`,
  manual: {
    title: "Persentase manual",
    help: "Saat data resmi tidak tersedia, isi persentase yang ditampilkan /usage di Claude Code atau di claude.ai. Nilainya dipakai sampai limit tersebut mungkin sudah reset.",
    save: "Atur",
    clear: "Hapus",
    invalid: "Isi angka dari 0 sampai 100.",
    failed: "Gagal menyimpan persentase manual. Coba lagi.",
    entered: (percent, age) => `Diisi ${percent}, ${age} lalu`,
  },
  windows: {
    fiveHour: "Jendela 5 jam",
    week: "Minggu ini",
    noUsageFiveHour: "Belum ada pemakaian dalam 5 jam terakhir.",
    noUsageWeek: "Belum ada pemakaian minggu ini.",
  },
  status: {
    estimate:
      "Estimasi dari log Claude Code lokal, tanpa pemakaian chat, desktop, atau mobile. Aktifkan mode akurat di Pengaturan untuk persentase resmi.",
    active: "Pemakaian resmi dari akun Claude Anda.",
    showingLast: "Menampilkan data resmi terakhir.",
    showingEstimates: "Menampilkan estimasi lokal.",
    reasons: {
      pending: "Menghubungkan ke akun Claude Anda.",
      signed_out: "Claude Code belum login di PC ini.",
      token_expired: "Login Claude Code sudah kedaluwarsa. Buka Claude Code untuk memperbaruinya.",
      unauthorized: "Akun Claude Anda menolak permintaan.",
      unavailable: "Pemakaian resmi sedang tidak tersedia.",
      retrying: "Tidak bisa menghubungi Claude. Mencoba lagi sebentar lagi.",
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
    alertHelp: "Warna tray berubah pada persentase limit ini.",
    warningAt: "Peringatan di",
    highAt: "Tinggi di",
    criticalAt: "Kritis di",
    refresh: "Perbarui pemakaian resmi",
    refreshHelp: "Dipakai saat mode akurat aktif.",
    interval: (minutes) => (minutes === 1 ? "Setiap menit" : `Setiap ${String(minutes)} menit`),
    language: "Bahasa",
    languageNames: { system: "Ikuti Windows", en: "English", id: "Bahasa Indonesia" },
    save: "Simpan",
    saved: "Tersimpan.",
    saveFailed: "Gagal menyimpan pengaturan. Coba lagi.",
    wholeNumbers: "Gunakan bilangan bulat 1 sampai 100.",
    rising: "Setiap level harus lebih tinggi dari level sebelumnya.",
  },
  accurate: {
    label: "Mode akurat",
    help: "Membaca login Claude Code di PC ini untuk menampilkan persentase resmi dari Anthropic. Token hanya ada di memori dan tidak pernah disimpan atau dikirim ke tempat lain.",
    confirmTitle: "Sebelum mengaktifkan mode akurat",
    risks: {
      token:
        "Plimsoll membaca token login yang disimpan Claude Code di PC ini dan hanya memakainya untuk meminta data pemakaian Anda ke Anthropic.",
      storage:
        "Token hanya ada di memori. Token tidak pernah ditulis ke disk, dicatat di log, atau dikirim ke tempat lain.",
      endpoint:
        "Endpoint pemakaian ini tidak terdokumentasi. Anthropic bisa mengubah atau memblokirnya kapan saja, dan Plimsoll akan kembali ke estimasi lokal.",
      unofficial:
        "Ini bukan fitur resmi Anthropic. Aktifkan hanya jika Anda nyaman dengan hal itu.",
      requests: (minutes) =>
        `Selama aktif, Plimsoll meminta data pemakaian ${minutes === 1 ? "setiap menit" : `setiap ${String(minutes)} menit`}. Anda bisa mematikannya kapan saja.`,
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
