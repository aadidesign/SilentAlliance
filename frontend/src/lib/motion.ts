/**
 * Shared motion variants for consistent page and element entrance animations.
 * All main-layout pages should use `pageEntrance` for their wrapper.
 * List items should use `listItemEntrance` with a capped stagger delay.
 */

export const pageEntrance = {
  initial: { opacity: 0, y: 8 },
  animate: { opacity: 1, y: 0 },
  transition: { duration: 0.25 },
};

export const listItemEntrance = (index: number) => ({
  initial: { opacity: 0, y: 10 },
  animate: { opacity: 1, y: 0 },
  transition: { duration: 0.3, delay: Math.min(index * 0.05, 0.3) },
});
