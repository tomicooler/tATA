#ifndef TATA_LIB_UTILS
#define TATA_LIB_UTILS

#include <ranges>
#include <string>

namespace Utils {

template <typename T>
void tokenize(T &container, const std::string &str,
              std::string_view delimiter) {
  for (const auto word : std::views::split(str, delimiter)) {
    container.push_back(std::string{word.begin(), word.end()});
  }
}

} // namespace Utils

namespace fmt {

std::string format(std::string_view text, std::string_view to) {
  std::string str{text};
  std::string_view from{"{}"};
  size_t start_pos = 0;
  while ((start_pos = str.find(from, start_pos)) != std::string::npos) {
    str.replace(start_pos, from.length(), to);
    start_pos += to.length();
  }
  return str;
}

} // namespace fmt

#endif // TATA_LIB_UTILS
