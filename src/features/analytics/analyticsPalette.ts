export type RootCategoryColor = {
  backgroundColor: string;
  borderColor: string;
};

export const ROOT_CATEGORY_ORDER = [
  "Деньги",
  "Здоровье",
  "Логика",
  "Психика",
  "Отношения",
  "Рутина",
  "Система",
] as const;

export const ROOT_CATEGORY_COLORS: Record<string, RootCategoryColor> = {
  Деньги: {
    backgroundColor: "rgba(237, 173, 121, 0.92)",
    borderColor: "rgba(203, 138, 87, 0.98)",
  },
  Здоровье: {
    backgroundColor: "rgba(138, 214, 158, 0.92)",
    borderColor: "rgba(98, 180, 121, 0.98)",
  },
  Логика: {
    backgroundColor: "rgba(118, 177, 234, 0.92)",
    borderColor: "rgba(78, 142, 204, 0.98)",
  },
  Психика: {
    backgroundColor: "rgba(206, 151, 205, 0.92)",
    borderColor: "rgba(177, 114, 176, 0.98)",
  },
  Отношения: {
    backgroundColor: "rgba(240, 153, 170, 0.92)",
    borderColor: "rgba(208, 114, 133, 0.98)",
  },
  Рутина: {
    backgroundColor: "rgba(193, 206, 118, 0.92)",
    borderColor: "rgba(161, 176, 79, 0.98)",
  },
  Система: {
    backgroundColor: "rgba(124, 199, 201, 0.92)",
    borderColor: "rgba(86, 166, 168, 0.98)",
  },
};

export const FALLBACK_ROOT_CATEGORY_COLOR: RootCategoryColor = {
  backgroundColor: "rgba(173, 184, 199, 0.88)",
  borderColor: "rgba(138, 150, 166, 0.96)",
};

export function normalizeRootCategoryLabel(label: string) {
  const normalizedKey = label
    .trim()
    .replace(/[.!?]+$/u, "")
    .toLowerCase();

  switch (normalizedKey) {
    case "деньги":
      return "Деньги";
    case "здоровье":
      return "Здоровье";
    case "логика":
      return "Логика";
    case "психика":
      return "Психика";
    case "отношения":
      return "Отношения";
    case "рутина":
      return "Рутина";
    case "система":
      return "Система";
    default:
      return label.trim().replace(/[.!?]+$/u, "");
  }
}

export function getRootCategoryColor(label: string): RootCategoryColor {
  return (
    ROOT_CATEGORY_COLORS[normalizeRootCategoryLabel(label)] ??
    FALLBACK_ROOT_CATEGORY_COLOR
  );
}
