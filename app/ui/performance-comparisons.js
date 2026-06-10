const imageBase = "./assets/performance-comparisons";

export const performanceComparisonScenes = [
  comparisonScene("swamp", "Swamp", "swamp"),
  comparisonScene("swamp2", "Swamp 2", "swamp2"),
  comparisonScene("old-camp-day", "Old Camp Day", "old-camp-day"),
  comparisonScene("old-camp-night", "Old Camp Night", "old-camp-night"),
  comparisonScene("old-camp-night2", "Old Camp Night 2", "old-camp-night2"),
];

function comparisonScene(id, label, filePrefix) {
  return {
    id,
    label,
    thumbnail: {
      alt: `${label} comparison preview`,
      src: `${imageBase}/${filePrefix}-balanced-cine.webp`,
    },
    before: {
      label: "Overdose",
      src: `${imageBase}/${filePrefix}-cine.webp`,
    },
    after: {
      label: "Balanced (Overdose)",
      src: `${imageBase}/${filePrefix}-balanced-cine.webp`,
    },
  };
}
