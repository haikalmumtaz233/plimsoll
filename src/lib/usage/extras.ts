import type { CreditsView, MoneyView } from "../api/usage";
import type { Messages } from "../i18n/messages";

export interface CreditsSummary {
  status: string;
  amount: string | null;
}

const KNOWN_MODELS: Readonly<Record<string, string>> = {
  opus: "Opus",
  sonnet: "Sonnet",
  haiku: "Haiku",
};

export function modelLabel(model: string): string {
  const known = KNOWN_MODELS[model];
  if (known !== undefined) {
    return known;
  }
  return model
    .split(/[_-]+/)
    .filter((word) => word !== "")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}

export function modelLimitTitle(model: string, messages: Messages): string {
  return messages.limits.model(modelLabel(model));
}

export function formatMoney(money: MoneyView, messages: Messages): string {
  return new Intl.NumberFormat(messages.intlLocale, {
    style: "currency",
    currency: money.currency,
    minimumFractionDigits: money.exponent,
    maximumFractionDigits: money.exponent,
  }).format(money.minor / 10 ** money.exponent);
}

function amountText(credits: CreditsView, messages: Messages): string | null {
  if (credits.used === null) {
    return null;
  }
  const used = formatMoney(credits.used, messages);
  if (credits.limit === null) {
    return messages.credits.used(used);
  }
  return messages.credits.usedOf(used, formatMoney(credits.limit, messages));
}

export function creditsSummary(credits: CreditsView, messages: Messages): CreditsSummary {
  return {
    status: messages.credits.states[credits.state],
    amount: amountText(credits, messages),
  };
}
