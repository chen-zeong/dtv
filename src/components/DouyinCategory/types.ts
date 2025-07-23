export interface DouyinCategory1 {
  title: string;
  href: string; // Used as an identifier, not a real link here
  subcategories: DouyinCategory2[];
}

export interface DouyinCategory2 {
  title: string;
  href: string; // Used as an identifier
}

export interface DouyinCategorySelectedEvent {
  type: 'cate2'; // Douyin only has 2 levels
  cate1Href: string;
  cate2Href: string;
  cate1Name: string;
  cate2Name: string;
} 