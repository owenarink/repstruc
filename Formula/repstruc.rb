class Repstruc < Formula
  include Language::Python::Virtualenv

  desc "Generate and maintain README structure blocks for every folder in a repository"
  homepage "https://github.com/owenarink/repstruc"
  url "https://github.com/owenarink/repstruc/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "3a52603c78ce438591c396dbe08e1c83be6c1db6a69c3a4099a62d63cb8114b0"
  license "MIT"

  depends_on "python@3.12"

  def install
    virtualenv_install_with_resources
  end

  test do
    mkdir "demo" do
      system bin/"repstruc", "."
      assert_predicate testpath/"demo/README.md", :exist?
    end
  end
end
