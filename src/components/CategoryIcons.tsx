import {
  ShoppingCart,
  Car,
  Briefcase,
  Film,
  ShoppingBag,
  Zap,
  Heart,
  BookOpen,
  Coffee,
  Plane,
  Home,
  Utensils,
  Wifi,
  Smartphone,
  Music,
  Gamepad2,
  Gift,
  Stethoscope,
  GraduationCap,
  Building2,
  PiggyBank,
  CreditCard,
  Wallet,
  Wrench,
  Bot,
  Code,
  Laptop,
  Tv,
  Dumbbell,
  Dog,
  Cat,
  Leaf,
  Shirt,
  Scissors,
  Bus,
  Train,
  Hotel,
  Wine,
  Landmark,
  Shield,
  Cloud,
  type LucideIcon,
} from "lucide-react";

export interface CategoryIconDef {
  name: string;
  icon: LucideIcon;
  label: string;
}

export const CATEGORY_ICONS: CategoryIconDef[] = [
  { name: "utensils", icon: Utensils, label: "Food" },
  { name: "car", icon: Car, label: "Car" },
  { name: "briefcase", icon: Briefcase, label: "Work" },
  { name: "film", icon: Film, label: "Film" },
  { name: "shopping-bag", icon: ShoppingBag, label: "Shopping" },
  { name: "zap", icon: Zap, label: "Bills" },
  { name: "heart", icon: Heart, label: "Health" },
  { name: "book-open", icon: BookOpen, label: "Education" },
  { name: "coffee", icon: Coffee, label: "Coffee" },
  { name: "plane", icon: Plane, label: "Travel" },
  { name: "home", icon: Home, label: "Home" },
  { name: "shopping-cart", icon: ShoppingCart, label: "Cart" },
  { name: "wifi", icon: Wifi, label: "Internet" },
  { name: "smartphone", icon: Smartphone, label: "Phone" },
  { name: "music", icon: Music, label: "Music" },
  { name: "gamepad-2", icon: Gamepad2, label: "Gaming" },
  { name: "gift", icon: Gift, label: "Gift" },
  { name: "stethoscope", icon: Stethoscope, label: "Medical" },
  { name: "graduation-cap", icon: GraduationCap, label: "School" },
  { name: "building-2", icon: Building2, label: "Office" },
  { name: "piggy-bank", icon: PiggyBank, label: "Savings" },
  { name: "credit-card", icon: CreditCard, label: "Credit" },
  { name: "wallet", icon: Wallet, label: "Wallet" },
  { name: "wrench", icon: Wrench, label: "Repairs" },
  { name: "bot", icon: Bot, label: "AI" },
  { name: "code", icon: Code, label: "Code" },
  { name: "laptop", icon: Laptop, label: "Laptop" },
  { name: "tv", icon: Tv, label: "Streaming" },
  { name: "dumbbell", icon: Dumbbell, label: "Fitness" },
  { name: "dog", icon: Dog, label: "Dog" },
  { name: "cat", icon: Cat, label: "Cat" },
  { name: "leaf", icon: Leaf, label: "Nature" },
  { name: "shirt", icon: Shirt, label: "Clothing" },
  { name: "scissors", icon: Scissors, label: "Grooming" },
  { name: "bus", icon: Bus, label: "Bus" },
  { name: "train", icon: Train, label: "Train" },
  { name: "hotel", icon: Hotel, label: "Hotel" },
  { name: "wine", icon: Wine, label: "Drinks" },
  { name: "landmark", icon: Landmark, label: "Taxes" },
  { name: "shield", icon: Shield, label: "Insurance" },
  { name: "cloud", icon: Cloud, label: "Cloud" },
];

export function CategoryIcon({
  name,
  size = 14,
  style,
}: {
  name: string;
  size?: number;
  style?: React.CSSProperties;
}) {
  const iconEntry = CATEGORY_ICONS.find((i) => i.name === name);
  if (iconEntry) {
    const Icon = iconEntry.icon;
    return <Icon size={size} style={style} />;
  }
  return null;
}
